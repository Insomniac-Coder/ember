//! `[LNT-6]` — what `ember lint` reports beyond `ember check`'s diagnostics, each at `warn` unless
//! the package's `[lints]` gives it another level (`[MAN-3]`): an unnecessary clone (`L2001`), a
//! large `Copy` value passed by value (`L2002`), a guard held past its last use (`L3002`), an
//! allocation in a hot loop (`L4001`), dynamic dispatch on a final type (`L4002`), a large list of
//! 64-bit numbers that fit 32 bits indexed in a hot loop (`L4003`), a copying conversion at the FFI
//! boundary (`L5002`) and a lock held across a call that may block (`L7001`). `L5001` reads the
//! declarations as written, in the driver; `L3001` is `[WK-6]`'s, which `check` reports.
//!
//! They read the MIR the checks ran on, before inlining moves one function's operations into
//! another. A loop the compiler judges hot is an innermost one, holding no other (ADR-241).

use std::collections::{HashMap, HashSet};

use ember_diag::codes::{self, Code};
use ember_diag::{Diagnostic, Severity, Sink};
use ember_hir::Effects;
use ember_mir::{Body, Builtin, CastKind, FuncRef, LocalId, LocalKind, Operand, ParameterMode, Place, Projection, Rvalue, StmtKind, Terminator};
use ember_types::{ClassOpenness, CommonTypes, Ty, TyKind, TypeTable};

use crate::long_access_lint::successors;
use crate::regions::place_type;

/// `[MAN-3]` — how loud a lint is.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LintLevel {
    Allow,
    Warn,
    Error,
}

impl LintLevel {
    pub fn parse(text: &str) -> Option<LintLevel> {
        match text {
            "allow" => Some(LintLevel::Allow),
            "warn" => Some(LintLevel::Warn),
            "error" | "deny" => Some(LintLevel::Error),
            _ => None,
        }
    }
}

/// `[LNT-6]`, `[MAN-3]` — each lint's level and `large_copy`'s threshold, as `[lints]` sets them.
#[derive(Clone, Copy, Debug)]
pub struct PerfLints {
    levels: [LintLevel; 9],
    /// `large_copy = { threshold = N }`: the bytes above which `L2002` reports a value.
    pub large_copy_threshold: u64,
}

impl Default for PerfLints {
    fn default() -> PerfLints {
        PerfLints { levels: [LintLevel::Warn; 9], large_copy_threshold: 256 }
    }
}

impl PerfLints {
    pub const CODES: [Code; 9] = [
        codes::L2001,
        codes::L2002,
        codes::L3002,
        codes::L4001,
        codes::L4002,
        codes::L4003,
        codes::L5001,
        codes::L5002,
        codes::L7001,
    ];

    /// Sets `code`'s level; false when `code` is none of these lints.
    pub fn set(&mut self, code: Code, level: LintLevel) -> bool {
        match PerfLints::CODES.iter().position(|known| *known == code) {
            Some(index) => {
                self.levels[index] = level;
                true
            }
            None => false,
        }
    }

    /// Reports `diagnostic` at its lint's level: not at all, as a warning, or as an error.
    pub fn emit(&self, sink: &mut Sink, mut diagnostic: Diagnostic) {
        let level = diagnostic
            .code
            .and_then(|code| PerfLints::CODES.iter().position(|known| *known == code))
            .map_or(LintLevel::Warn, |index| self.levels[index]);
        match level {
            LintLevel::Allow => {}
            LintLevel::Warn => sink.emit(diagnostic),
            LintLevel::Error => {
                diagnostic.severity = Severity::Error;
                sink.emit(diagnostic);
            }
        }
    }
}

/// `[LNT-6]` — every lint above over the program's own functions (`standard[i]` says whether body
/// `i` is the standard library's).
pub fn lint_performance_all(
    bodies: &[Body],
    types: &TypeTable,
    common: &CommonTypes,
    standard: &[bool],
    settings: &PerfLints,
    sink: &mut Sink,
) {
    let effects = crate::effects::operation_effects(bodies, types);
    let returns = crate::range_facts::returned_arguments(bodies);
    let fields = crate::range_facts::field_element_ranges(bodies, types, common, &returns);
    let by_symbol: HashMap<&str, usize> = bodies.iter().enumerate().map(|(i, body)| (body.symbol.as_str(), i)).collect();
    let callee = |func: &FuncRef| match func {
        FuncRef::Direct { symbol, .. } => by_symbol.get(symbol.as_str()).copied(),
        _ => None,
    };
    for (index, body) in bodies.iter().enumerate() {
        if standard[index] || body.span.is_dummy() || body.emit_if_used || body.contracts.foreign.is_some() {
            continue;
        }
        let mut lint = Lint { body, types, settings, sink: &mut *sink };
        lint.unnecessary_clones(&|func| callee(func).map(|i| &bodies[i]));
        lint.large_copies();
        lint.guards(&effects.calls[index]);
        lint.hot_allocations(&effects.calls[index], &effects.stmts[index]);
        lint.final_dispatch();
        for narrow in crate::range_facts::narrow_lists_in_hot_loops(body, types, common, &returns, &fields) {
            let name = local_name(body, narrow.list);
            let TyKind::Vec { elem, .. } = *types.kind(body.local(narrow.list).ty) else { continue };
            let elem = types.display(elem);
            lint.emit(
                Diagnostic::lint(
                    codes::L4003,
                    narrow.span,
                    format!("`{name}` holds more than 64 KiB of `{elem}` values that all fit in `{}`, and this hot loop indexes it", narrow.narrow),
                )
                .primary_label("indexed on every turn")
                .help(format!(
                    "make it an `Array[{0}]` and widen each value as it is read (`{name}[i] as {elem}`): half the memory, so half the cache traffic",
                    narrow.narrow
                )),
            );
        }
        lint.ffi_copies(&|func| callee(func), bodies, standard);
    }
}

struct Lint<'a> {
    body: &'a Body,
    types: &'a TypeTable,
    settings: &'a PerfLints,
    sink: &'a mut Sink,
}

impl Lint<'_> {
    fn emit(&mut self, diagnostic: Diagnostic) {
        self.settings.emit(self.sink, diagnostic);
    }

    /// `L2001` — `x.clone()` of a value this function owns and never uses again: moving it is free.
    fn unnecessary_clones<'b>(&mut self, callee: &dyn Fn(&FuncRef) -> Option<&'b Body>) {
        let body = self.body;
        for block in &body.blocks {
            let Terminator::Call { func, args, next, .. } = &block.terminator else { continue };
            let is_clone = match func {
                FuncRef::Builtin { which: Builtin::ArrayClone { .. } | Builtin::CloneParts { .. }, .. } => true,
                FuncRef::Direct { .. } => callee(func).is_some_and(|target| target.name == "clone" || target.name.ends_with(".clone")),
                _ => false,
            };
            let Some(Operand::Copy(place) | Operand::Move(place)) = args.first() else { continue };
            if !is_clone || !place.projection.is_empty() || block.terminator_span.is_dummy() {
                continue;
            }
            let local = place.local;
            let decl = body.local(local);
            let owned = match decl.kind {
                LocalKind::User => true,
                LocalKind::Arg => body.param_modes.get(local.0 as usize - 1) == Some(&ParameterMode::Owned),
                _ => false,
            };
            let viewed = matches!(self.types.kind(decl.ty), TyKind::Ref { .. } | TyKind::Ptr { .. } | TyKind::Span { .. } | TyKind::Str);
            if !owned || viewed || decl.name.is_none() || self.types.is_copy(decl.ty) {
                continue;
            }
            if used_after(body, next.0 as usize, &aliases(body, self.types, local)) {
                continue;
            }
            let name = local_name(body, local);
            self.emit(
                Diagnostic::lint(codes::L2001, block.terminator_span, format!("`{name}` is cloned, then never used again"))
                    .primary_label("this copy is not needed")
                    .help(format!("pass `{name}` itself: nothing reads it after this, so moving it costs nothing")),
            );
        }
    }

    /// `L2002` — an `owned` parameter of a `Copy` type larger than `large_copy`'s threshold: every
    /// call copies it in, where a borrowed one is passed by address (`[BRW-8]`).
    fn large_copies(&mut self) {
        let body = self.body;
        for (position, mode) in body.param_modes.iter().enumerate() {
            let local = LocalId(position as u32 + 1);
            let decl = body.local(local);
            if *mode != ParameterMode::Owned || !self.types.is_copy(decl.ty) || decl.span.is_dummy() {
                continue;
            }
            let size = self.types.layout(decl.ty).size;
            if size <= self.settings.large_copy_threshold {
                continue;
            }
            let name = local_name(body, local);
            self.emit(
                Diagnostic::lint(codes::L2002, decl.span, format!("`{name}` is a {size}-byte `Copy` value passed by value"))
                    .primary_label("copied in by every call")
                    .help(format!("take `{name}` borrowed (without `owned`): it is then passed by address and read in place"))
                    .note(format!(
                        "the lint reports `Copy` values larger than `large_copy = {{ threshold = N }}` bytes in `[lints]`, here {}",
                        self.settings.large_copy_threshold
                    )),
            );
        }
    }

    /// `L3002` and `L7001` — a lock guard (`Mutex`, `RwLock`) held across calls: past its last use,
    /// and across one that may block (`calls[b]` is the call ending block `b`'s known effects).
    fn guards(&mut self, calls: &HashMap<usize, Effects>) {
        let body = self.body;
        for (index, decl) in body.locals.iter().enumerate() {
            let guard = LocalId(index as u32);
            if !is_lock_guard(self.types, decl.ty) {
                continue;
            }
            let Some((start, at, span)) = only_whole_write(body, guard) else { continue };
            let aliases = aliases(body, self.types, guard);
            let name = local_name(body, guard);
            // Where the guard is last used in the source: a call past it is one the guard need not
            // be held across (the second `print` of a `println(g)` is part of its last use).
            let last_use = use_spans(body, &aliases).into_iter().filter(|used| used.file == span.file).map(|used| used.end).max();
            let mut past_last_use = false;
            for call in held_calls(body, guard, start, at) {
                let block = &body.blocks[call];
                let call_span = block.terminator_span;
                if call_span.is_dummy() {
                    continue;
                }
                if calls.get(&call).is_some_and(|effects| effects.contains(Effects::BLOCK)) {
                    self.emit(
                        Diagnostic::lint(codes::L7001, call_span, format!("the lock `{name}` guards is held across a call that may block"))
                            .primary_label("this call may wait")
                            .secondary(span, "the lock is taken here")
                            .help(format!("release `{name}` before the call (end its `with` block), so other threads are not kept waiting too")),
                    );
                }
                let Terminator::Call { args, func, next, .. } = &block.terminator else { continue };
                let uses = args.iter().chain(match func {
                    FuncRef::Indirect { operand, .. } => Some(operand),
                    _ => None,
                })
                .any(|operand| operand_mentions(operand, &aliases));
                let after_in_source = call_span.file == span.file && last_use.is_none_or(|end| call_span.start >= end);
                if !past_last_use && !uses && after_in_source && !used_after(body, next.0 as usize, &aliases) {
                    past_last_use = true;
                    self.emit(
                        Diagnostic::lint(codes::L3002, call_span, format!("`{name}` is still held here, after its last use"))
                            .primary_label("the guard is held across this call")
                            .secondary(span, "taken here")
                            .help(format!("end the `with` block, or the scope holding `{name}`, after its last use")),
                    );
                }
            }
        }
    }

    /// `L4001` — a call or operation in a hot loop that makes a new value owning heap memory.
    fn hot_allocations(&mut self, calls: &HashMap<usize, Effects>, stmts: &HashMap<(usize, usize), Effects>) {
        let body = self.body;
        let mut reported: HashSet<ember_span::Span> = HashSet::new();
        for (_, blocks) in crate::range_facts::innermost_loops(body) {
            let mut blocks: Vec<usize> = blocks.into_iter().collect();
            blocks.sort_unstable();
            for block_index in blocks {
                let block = &body.blocks[block_index];
                let owns = |place: &Place| self.types.needs_drop(place_type(body, self.types, place));
                let mut sites: Vec<ember_span::Span> = Vec::new();
                for (stmt_index, stmt) in block.stmts.iter().enumerate() {
                    if let StmtKind::Assign { place, .. } = &stmt.kind
                        && stmts.get(&(block_index, stmt_index)).is_some_and(|effects| effects.contains(Effects::ALLOC))
                        && owns(place)
                    {
                        sites.push(stmt.span);
                    }
                }
                if let Terminator::Call { dest, .. } = &block.terminator
                    && calls.get(&block_index).is_some_and(|effects| effects.contains(Effects::ALLOC))
                    && owns(dest)
                {
                    sites.push(block.terminator_span);
                }
                for span in sites {
                    if span.is_dummy() || !reported.insert(span) {
                        continue;
                    }
                    self.emit(
                        Diagnostic::lint(codes::L4001, span, "allocation in a hot loop")
                            .primary_label("this allocates on every turn")
                            .help("make the value once, before the loop, and reuse it (`clear()` empties a collection and keeps its storage)")
                            .note("an innermost loop is hot: it holds no other loop [LNT-6]"),
                    );
                }
            }
        }
    }

    /// `L4002` — a call through `dyn` whose receiver always holds one final type: the value was
    /// converted to `dyn` here, from a struct, an enum or a class nothing derives from.
    fn final_dispatch(&mut self) {
        let body = self.body;
        for block in &body.blocks {
            let Terminator::Call { func: FuncRef::Interface { interface, .. }, args, .. } = &block.terminator else { continue };
            let Some(Operand::Copy(receiver) | Operand::Move(receiver)) = args.first() else { continue };
            if !receiver.projection.is_empty() || block.terminator_span.is_dummy() {
                continue;
            }
            let Some(concrete) = upcast_source(body, receiver.local) else { continue };
            let concrete = match *self.types.kind(concrete) {
                TyKind::Ref { inner, .. } => inner,
                _ => concrete,
            };
            let is_final = match self.types.kind(concrete) {
                TyKind::Class(id) => self.types.class_def(*id).openness == ClassOpenness::Final,
                TyKind::Struct(_) | TyKind::Enum(_) | TyKind::Tuple(_) | TyKind::Int(_) | TyKind::Uint(_) | TyKind::Float(_) => true,
                _ => false,
            };
            if !is_final {
                continue;
            }
            let ty = self.types.display(concrete);
            self.emit(
                Diagnostic::lint(
                    codes::L4002,
                    block.terminator_span,
                    format!("this `{interface}` call is dispatched dynamically, but its receiver is always a `{ty}`, a final type"),
                )
                .primary_label("a call through the interface's table")
                .help(format!("call the method on the `{ty}` itself: the call is then direct, and can be inlined")),
            );
        }
    }

    /// `L5002` — a standard conversion that copies (`to_cstring`, `to_wide`, `WideString.from_units`,
    /// `WideString.to_string`) whose result is passed to a foreign function, or whose argument is
    /// what one returned.
    fn ffi_copies(&mut self, callee: &dyn Fn(&FuncRef) -> Option<usize>, bodies: &[Body], standard: &[bool]) {
        // The standard functions, by how their paths end (`[MNG-1]`): `str.to_cstring` and
        // `String.to_cstring`, `str.to_wide`, `WideString.from_units`, `WideString.to_string`.
        const COPYING: [&[&str]; 4] = [&["to_cstring"], &["str", "to_wide"], &["WideString", "from_units"], &["WideString", "to_string"]];
        let copying = |symbol: &str| {
            ember_branding::demangle(symbol).is_some_and(|found| {
                let path = &found.components;
                COPYING.iter().any(|tail| {
                    path.len() >= tail.len() && path[path.len() - tail.len()..].iter().zip(tail.iter()).all(|(a, b)| a == b)
                })
            })
        };
        let body = self.body;
        let is_foreign = |func: &FuncRef| callee(func).is_some_and(|i| bodies[i].contracts.foreign.is_some());
        let foreign_results: Vec<LocalId> = body
            .blocks
            .iter()
            .filter_map(|block| match &block.terminator {
                Terminator::Call { func, dest, .. } if is_foreign(func) => Some(dest.local),
                _ => None,
            })
            .collect();
        let from_c = flows_from(body, &foreign_results, &is_foreign);
        for block in &body.blocks {
            let Terminator::Call { func, args, dest, .. } = &block.terminator else { continue };
            let Some(target) = callee(func).filter(|&i| standard[i]) else { continue };
            if block.terminator_span.is_dummy() || !copying(&bodies[target].symbol) {
                continue;
            }
            let name = &bodies[target].name;
            let method = name.rsplit('.').next().unwrap_or(name);
            let to_c = flows_from(body, &[dest.local], &is_foreign);
            let passed = body.blocks.iter().find(|other| match &other.terminator {
                Terminator::Call { func, args, .. } => is_foreign(func) && args.iter().any(|arg| operand_mentions(arg, &to_c)),
                _ => false,
            });
            let returned = args.iter().any(|arg| operand_mentions(arg, &from_c));
            let diagnostic = match (passed, returned) {
                (Some(call), _) => Diagnostic::lint(codes::L5002, block.terminator_span, format!("`{method}` copies its argument for a foreign call"))
                    .primary_label("a copy made here")
                    .secondary(call.terminator_span, "passed to C here"),
                (None, true) => Diagnostic::lint(codes::L5002, block.terminator_span, format!("`{method}` copies what a foreign call returned"))
                    .primary_label("a copy made here"),
                (None, false) => continue,
            };
            self.emit(diagnostic.help("convert once and keep the converted value when the same text crosses again"));
        }
    }
}

/// A local's name as written, or `_N` for a temporary.
fn local_name(body: &Body, local: LocalId) -> String {
    body.local(local).name.clone().unwrap_or_else(|| format!("_{}", local.0))
}

/// Whether `ty` is a `std.sync` lock guard, whose end releases the lock.
fn is_lock_guard(types: &TypeTable, ty: Ty) -> bool {
    matches!(types.kind(ty), TyKind::Struct(id)
        if types.struct_def(*id).origin.as_ref().is_some_and(|(origin, _)|
            matches!(origin.as_str(), "std.sync.MutexGuard" | "std.sync.RwLockReadGuard" | "std.sync.RwLockWriteGuard")))
}

/// The one statement writing all of `local`: its block, its index and its span.
fn only_whole_write(body: &Body, local: LocalId) -> Option<(usize, usize, ember_span::Span)> {
    let mut found = None;
    for (b, block) in body.blocks.iter().enumerate() {
        for (s, stmt) in block.stmts.iter().enumerate() {
            if let StmtKind::Assign { place, .. } = &stmt.kind
                && place.local == local
            {
                if !place.projection.is_empty() || found.is_some() {
                    return None;
                }
                found = Some((b, s, stmt.span));
            }
        }
        if matches!(&block.terminator, Terminator::Call { dest, .. } if dest.local == local) {
            return None;
        }
    }
    found
}

/// The blocks whose call runs while `guard` is held: from its write (statement `at` of `start`) to
/// its drop, on every path.
fn held_calls(body: &Body, guard: LocalId, start: usize, at: usize) -> Vec<usize> {
    let mut calls = Vec::new();
    let mut seen: HashSet<usize> = HashSet::new();
    let mut work = vec![(start, at + 1)];
    while let Some((block_index, from)) = work.pop() {
        let block = &body.blocks[block_index];
        let ended = block.stmts[from.min(block.stmts.len())..].iter().any(|stmt| {
            matches!(&stmt.kind, StmtKind::Drop { place, .. } if place.local == guard && place.projection.is_empty())
                || matches!(&stmt.kind, StmtKind::StorageDead(local) if *local == guard)
        });
        if ended {
            continue;
        }
        if matches!(block.terminator, Terminator::Call { .. }) && !calls.contains(&block_index) {
            calls.push(block_index);
        }
        for next in successors(&block.terminator) {
            if seen.insert(next) {
                work.push((next, 0));
            }
        }
    }
    calls.sort_unstable();
    calls
}

/// `local` and every local that may hold a view of it: a reference to it or into it, a copy of
/// one, and a call's result made from one, each of a type that holds a view (`[TYP-14]`) or a
/// raw pointer.
fn aliases(body: &Body, types: &TypeTable, local: LocalId) -> HashSet<LocalId> {
    let mut set = HashSet::from([local]);
    let holds_no_view = |place: &Place| {
        let ty = place_type(body, types, place);
        !types.is_view(ty) && !matches!(types.kind(ty), TyKind::Ptr { .. })
    };
    loop {
        let before = set.len();
        for block in &body.blocks {
            for stmt in &block.stmts {
                if let StmtKind::Assign { place, rvalue } = &stmt.kind
                    && rvalue_mentions(rvalue, &set)
                    && !holds_no_view(place)
                {
                    set.insert(place.local);
                }
            }
            if let Terminator::Call { args, dest, .. } = &block.terminator
                && args.iter().any(|arg| operand_mentions(arg, &set))
                && !holds_no_view(dest)
            {
                set.insert(dest.local);
            }
        }
        if set.len() == before {
            return set;
        }
    }
}

/// The locals whose value may come from one of `seeds`: assigned from one, or the result of a call
/// given one (a foreign call's result excepted, which C makes).
fn flows_from(body: &Body, seeds: &[LocalId], is_foreign: &dyn Fn(&FuncRef) -> bool) -> HashSet<LocalId> {
    let mut set: HashSet<LocalId> = seeds.iter().copied().collect();
    loop {
        let before = set.len();
        for block in &body.blocks {
            for stmt in &block.stmts {
                if let StmtKind::Assign { place, rvalue } = &stmt.kind
                    && rvalue_mentions(rvalue, &set)
                {
                    set.insert(place.local);
                }
            }
            if let Terminator::Call { func, args, dest, .. } = &block.terminator
                && !is_foreign(func)
                && args.iter().any(|arg| operand_mentions(arg, &set))
            {
                set.insert(dest.local);
            }
        }
        if set.len() == before {
            return set;
        }
    }
}

/// Whether any of `locals` is read from block `start` on, on some path, before each is written
/// whole again or ends (a drop or the end of its storage is no use).
fn used_after(body: &Body, start: usize, locals: &HashSet<LocalId>) -> bool {
    let mut seen: HashSet<usize> = HashSet::new();
    let mut work = vec![start];
    while let Some(block_index) = work.pop() {
        if !seen.insert(block_index) {
            continue;
        }
        let block = &body.blocks[block_index];
        let mut ended = false;
        for stmt in &block.stmts {
            match &stmt.kind {
                StmtKind::Drop { place, flag, .. } => {
                    if flag.is_some_and(|flag| locals.contains(&flag)) || place.projection.iter().any(|p| matches!(p, Projection::Index(i) if locals.contains(i))) {
                        return true;
                    }
                    if locals.contains(&place.local) && place.projection.is_empty() {
                        ended = true;
                        break;
                    }
                }
                StmtKind::StorageDead(local) if locals.contains(local) => {
                    ended = true;
                    break;
                }
                StmtKind::StorageLive(_) | StmtKind::StorageDead(_) | StmtKind::Nop => {}
                StmtKind::Assign { place, rvalue } => {
                    if rvalue_mentions(rvalue, locals) || place_reads(place, locals) {
                        return true;
                    }
                    if locals.contains(&place.local) && place.projection.is_empty() {
                        ended = true;
                        break;
                    }
                }
                StmtKind::CheckedBinaryOp { dest, overflow, lhs, rhs, .. } => {
                    if [dest, overflow].iter().any(|place| place_mentions(place, locals))
                        || operand_mentions(lhs, locals)
                        || operand_mentions(rhs, locals)
                    {
                        return true;
                    }
                }
                StmtKind::BeginAccess { place, .. }
                | StmtKind::BeginAccessTransfer { place, .. }
                | StmtKind::EndAccess { place, .. }
                | StmtKind::EndAccessTransfer { place, .. }
                | StmtKind::LendAccess { place, .. } => {
                    if place_mentions(place, locals) {
                        return true;
                    }
                }
            }
        }
        if ended {
            continue;
        }
        match &block.terminator {
            Terminator::Call { func, args, dest, .. } => {
                let callee = match func {
                    FuncRef::Indirect { operand, .. } => operand_mentions(operand, locals),
                    _ => false,
                };
                if callee || args.iter().any(|arg| operand_mentions(arg, locals)) || place_reads(dest, locals) {
                    return true;
                }
                if locals.contains(&dest.local) && dest.projection.is_empty() {
                    continue;
                }
            }
            Terminator::SwitchInt { discr, .. } => {
                if operand_mentions(discr, locals) {
                    return true;
                }
            }
            Terminator::Assert { cond, msg, .. } => {
                let extra: Vec<&Operand> = match msg {
                    ember_mir::AssertKind::Bounds { len, index } => vec![len, index],
                    ember_mir::AssertKind::RefCellBorrow { file, line } => vec![file, line],
                    ember_mir::AssertKind::Panic { message } => vec![message],
                    _ => Vec::new(),
                };
                if operand_mentions(cond, locals) || extra.into_iter().any(|operand| operand_mentions(operand, locals)) {
                    return true;
                }
            }
            Terminator::Goto(_) | Terminator::Return | Terminator::Unreachable => {}
        }
        work.extend(successors(&block.terminator));
    }
    false
}

/// The spans of the statements and calls that read any of `locals` (a drop, or the end of its
/// storage, is no read).
fn use_spans(body: &Body, locals: &HashSet<LocalId>) -> Vec<ember_span::Span> {
    let mut spans = Vec::new();
    for block in &body.blocks {
        for stmt in &block.stmts {
            let reads = match &stmt.kind {
                StmtKind::Assign { place, rvalue } => rvalue_mentions(rvalue, locals) || place_reads(place, locals),
                StmtKind::CheckedBinaryOp { dest, overflow, lhs, rhs, .. } => {
                    place_mentions(dest, locals) || place_mentions(overflow, locals) || operand_mentions(lhs, locals) || operand_mentions(rhs, locals)
                }
                _ => false,
            };
            if reads && !stmt.span.is_dummy() {
                spans.push(stmt.span);
            }
        }
        let reads = match &block.terminator {
            Terminator::Call { args, .. } => args.iter().any(|arg| operand_mentions(arg, locals)),
            Terminator::SwitchInt { discr, .. } => operand_mentions(discr, locals),
            Terminator::Assert { cond, .. } => operand_mentions(cond, locals),
            _ => false,
        };
        if reads && !block.terminator_span.is_dummy() {
            spans.push(block.terminator_span);
        }
    }
    spans
}

/// The `dyn` conversion a local holds, followed through copies: the type it converted from.
fn upcast_source(body: &Body, mut local: LocalId) -> Option<Ty> {
    for _ in 0..8 {
        if body.local(local).kind == LocalKind::Arg {
            return None;
        }
        let (block, stmt, _) = only_whole_write(body, local)?;
        let StmtKind::Assign { rvalue, .. } = &body.blocks[block].stmts[stmt].kind else { return None };
        match rvalue {
            Rvalue::Cast { kind: CastKind::InterfaceUpcast { concrete, .. } | CastKind::ClassInterfaceUpcast { concrete, .. }, .. } => {
                return Some(*concrete);
            }
            Rvalue::Use(Operand::Copy(from) | Operand::Move(from)) if from.projection.is_empty() => local = from.local,
            _ => return None,
        }
    }
    None
}

/// Whether a place written reads any of `locals`: an index, or a reference written through.
fn place_reads(place: &Place, locals: &HashSet<LocalId>) -> bool {
    place.projection.iter().any(|projection| match projection {
        Projection::Index(index) => locals.contains(index),
        Projection::Deref => locals.contains(&place.local),
        _ => false,
    }) || (!place.projection.is_empty() && locals.contains(&place.local))
}

fn place_mentions(place: &Place, locals: &HashSet<LocalId>) -> bool {
    locals.contains(&place.local) || place.projection.iter().any(|p| matches!(p, Projection::Index(i) if locals.contains(i)))
}

fn operand_mentions(operand: &Operand, locals: &HashSet<LocalId>) -> bool {
    match operand {
        Operand::Copy(place) | Operand::Move(place) => place_mentions(place, locals),
        Operand::Const(_) => false,
    }
}

fn rvalue_mentions(rvalue: &Rvalue, locals: &HashSet<LocalId>) -> bool {
    match rvalue {
        Rvalue::Use(operand) | Rvalue::UnaryOp { operand, .. } | Rvalue::Cast { operand, .. } => operand_mentions(operand, locals),
        Rvalue::BinaryOp { lhs, rhs, .. } => operand_mentions(lhs, locals) || operand_mentions(rhs, locals),
        Rvalue::Aggregate { operands, .. } => operands.iter().any(|operand| operand_mentions(operand, locals)),
        Rvalue::Repeat { value, .. } => operand_mentions(value, locals),
        Rvalue::Discriminant(place) | Rvalue::Ref { place, .. } => place_mentions(place, locals),
    }
}
