//! `[EFF-1]`–`[EFF-6]`, `[EFF-15]`–`[EFF-21]`, `[DET-1]` — every function's effect set, computed
//! from its body and its callees' over the call graph, and its contract attributes checked against
//! it, naming the chain to the offending operation (`E4001`, `E4002`, `E4003`, `E4040`, `E4041`,
//! `E4042`, `E4070`).
//!
//! The verdict is taken once, after the profile-independent proofs and before inlining, as
//! `@static_safe`'s is (`[EFF-15]`), so it is the same in every profile. By then a known function
//! passed to a callable parameter has its own copy of the callee that calls it directly, so such a
//! call contributes the effects of the function actually passed (`[EFF-2]`). A call through a
//! function value or a `dyn` interface contributes every effect: no function type declares its
//! effects yet.
//!
//! An operation's effects are its own and those of everything it runs where it runs (`[EFF-1]`):
//! the functions it calls, and the drops it performs (a type's `drop`, a class's when the last
//! handle goes, with each class below the handle's type). A recursive group gets the union of its
//! members, which the fixed point below gives.

use std::collections::{HashMap, HashSet, VecDeque};

use ember_diag::{Diagnostic, Sink, codes};
use ember_hir::{Builtin, Effects, FloatLib};
use ember_mir::{AggregateKind, AssertKind, Body, FuncRef, Operand, Rvalue, StmtKind, Terminator};
use ember_span::Span;
use ember_types::{ClassId, Ty, TyKind, TypeTable};

use crate::regions::place_type;

/// Where an effect enters one body: an operation of its own, or a call into another body.
#[derive(Clone, Copy)]
enum Origin {
    Operation(&'static str),
    Call(usize),
}

#[derive(Clone, Copy)]
struct Site {
    effects: Effects,
    span: Span,
    origin: Origin,
}

/// `[JOB-1]` — whether a job's state of type `ty` surely fits its slot: 64 bytes, aligned to at most
/// 16, by C's layout. A type whose size is not worked out here does not.
fn job_state_fits(types: &TypeTable, ty: Ty) -> bool {
    layout(types, ty).is_some_and(|(size, align)| size <= 64 && align <= 16)
}

/// C's size and alignment of a value of `ty`, where they follow from its parts alone.
fn layout(types: &TypeTable, ty: Ty) -> Option<(u64, u64)> {
    use ember_types::{FloatTy, IntTy, UintTy};
    let record = |parts: &mut dyn Iterator<Item = Ty>| -> Option<(u64, u64)> {
        let (mut size, mut align) = (0u64, 1u64);
        for part in parts {
            let (part_size, part_align) = layout(types, part)?;
            size = size.div_ceil(part_align) * part_align + part_size;
            align = align.max(part_align);
        }
        Some((size.div_ceil(align) * align, align))
    };
    Some(match types.kind(ty) {
        TyKind::Void => (0, 1),
        TyKind::Bool | TyKind::Int(IntTy::I8) | TyKind::Uint(UintTy::U8) => (1, 1),
        TyKind::Int(IntTy::I16) | TyKind::Uint(UintTy::U16) | TyKind::Float(FloatTy::F16) => (2, 2),
        TyKind::Int(IntTy::I32) | TyKind::Uint(UintTy::U32) | TyKind::Float(FloatTy::F32) | TyKind::Char => (4, 4),
        TyKind::Int(IntTy::I64 | IntTy::Isize) | TyKind::Uint(UintTy::U64 | UintTy::Usize) | TyKind::Float(FloatTy::F64) => (8, 8),
        TyKind::Ref { .. } | TyKind::Ptr { .. } | TyKind::Class(_) | TyKind::ClassInterface(_) => (8, 8),
        TyKind::Str | TyKind::Span { .. } => (16, 8),
        TyKind::Vec { .. } => (24, 8),
        TyKind::Array { elem, len } => {
            let (size, align) = layout(types, *elem)?;
            (size.checked_mul(*len)?, align)
        }
        TyKind::Tuple(items) => record(&mut items.iter().copied())?,
        TyKind::Struct(id) => record(&mut types.struct_def(*id).fields.iter().map(|field| field.ty))?,
        _ => return None,
    })
}

/// Each body's effect set, in `bodies`' order (`[EFF-1]`).
pub fn effect_sets(bodies: &[Body], types: &TypeTable) -> Vec<Effects> {
    Graph::new(bodies, types).effects
}

/// `[CT-1]`, `[CT-2]` — what may not run at compile time: anything beyond `Alloc`, `Panic` and the
/// run-time checks.
pub const COMPTIME_FORBIDDEN: Effects =
    Effects(Effects::SYNC.0 | Effects::LOCK.0 | Effects::BLOCK.0 | Effects::IO.0 | Effects::UNSAFE.0 | Effects::FFI.0 | Effects::NONDET.0);

/// `[CT-2]` — for the body at `root`, the first effect compile time cannot run that it reaches, the
/// operation that has it and the call chain from `root` to it (`E6010` names them).
pub fn comptime_forbidden(bodies: &[Body], types: &TypeTable, root: usize) -> Option<(Effects, Vec<String>)> {
    let graph = Graph::new(bodies, types);
    let found = graph.effects[root] & COMPTIME_FORBIDDEN;
    let (effect, _) = found.each().next()?;
    let (_, _, chain) = graph.chain(root, effect, types)?;
    Some((effect, chain))
}

/// `[PHIL-2]`, `[CLO-10]` — one place a function of the program allocates.
pub struct AllocationSite {
    /// The function, as the program names it.
    pub function: String,
    pub span: Span,
    /// What allocates there, in the order the function runs it.
    pub operations: Vec<String>,
}

/// `[PHIL-2]`, `[CLO-10]` — every place a function the program wrote allocates, for `ember inspect
/// --alloc`: each operation of its own that carries `Alloc`, and each call into a standard-library
/// function that allocates on its own (a collection's growth: `Map.insert`). `standard[i]` says
/// whether body `i` is the standard library's, `standard_file` whether a file is its source.
///
/// A call through a function value or a `dyn` value, whose effects nothing declares, is no site:
/// what it runs is listed where that is written. A site written in the standard library (a field
/// default of a standard type, evaluated where the program constructs one) is placed at the next
/// place of the program's own that the function reaches: the construction it is part of. A site
/// inside another of the same function's is part of it (a list literal's elements pushed, an
/// f-string's pieces), so each site is one expression as written, with everything it allocates.
pub fn allocation_sites(
    bodies: &[Body],
    types: &TypeTable,
    standard: &[bool],
    standard_file: &dyn Fn(ember_span::FileId) -> bool,
) -> Vec<AllocationSite> {
    let index: HashMap<&str, usize> =
        bodies.iter().enumerate().map(|(i, body)| (body.symbol.as_str(), i)).collect();
    let mut builder = SiteBuilder { bodies, types, index: &index, ancestors: class_ancestors(types), drops: HashMap::new() };
    // What each standard function allocates by itself: through its own operations and the
    // standard functions it calls, never through the program's functions it is given.
    let known: Vec<Vec<Site>> = bodies
        .iter()
        .map(|body| {
            let mut sites = builder.sites(body);
            sites.retain(|site| match site.origin {
                Origin::Operation(_) => site.effects != Effects::ALL,
                Origin::Call(callee) => standard[callee],
            });
            sites
        })
        .collect();
    let allocates = fixed_point(&known);
    let own_file = |span: Span| span != Span::DUMMY && !standard_file(span.file);
    let names = function_names(bodies, types, standard);
    let mut out: Vec<AllocationSite> = Vec::new();
    let mut at: HashMap<(String, Span), usize> = HashMap::new();
    for (index, body) in bodies.iter().enumerate() {
        if standard[index] {
            continue;
        }
        let function = names[index].clone();
        for (block_index, block) in body.blocks.iter().enumerate() {
            for position in 0..=block.stmts.len() {
                let mut found = Vec::new();
                match block.stmts.get(position) {
                    Some(stmt) => builder.stmt_sites(body, stmt, &mut found),
                    None => builder.terminator_sites(body, block, &mut found),
                }
                for site in found {
                    let operation = match site.origin {
                        Origin::Operation(operation)
                            if site.effects.contains(Effects::ALLOC) && site.effects != Effects::ALL =>
                        {
                            operation.to_string()
                        }
                        Origin::Call(callee) if standard[callee] && allocates[callee].contains(Effects::ALLOC) => {
                            format!("`{}`", method_name(&bodies[callee], types))
                        }
                        _ => continue,
                    };
                    let span = if own_file(site.span) {
                        site.span
                    } else {
                        own_place_after(body, block_index, position, &own_file).unwrap_or(body.span)
                    };
                    let entry = *at.entry((function.clone(), span)).or_insert_with(|| {
                        out.push(AllocationSite { function: function.clone(), span, operations: Vec::new() });
                        out.len() - 1
                    });
                    if !out[entry].operations.contains(&operation) {
                        out[entry].operations.push(operation);
                    }
                }
            }
        }
    }
    out.sort_by_key(|site| (site.span.file, site.span.start, std::cmp::Reverse(site.span.end)));
    let mut merged: Vec<AllocationSite> = Vec::new();
    for site in out {
        let outer = merged.iter_mut().find(|outer| {
            outer.function == site.function
                && outer.span.file == site.span.file
                && outer.span.start <= site.span.start
                && site.span.end <= outer.span.end
        });
        match outer {
            Some(outer) => {
                for operation in site.operations {
                    if !outer.operations.contains(&operation) {
                        outer.operations.push(operation);
                    }
                }
            }
            None => merged.push(site),
        }
    }
    merged
}

/// Each function's name as the program shows it: a lambda is named after the function that makes
/// it, the program's own one that names it rather than a standard function given it to call
/// ("lambda in f").
fn function_names(bodies: &[Body], types: &TypeTable, standard: &[bool]) -> Vec<String> {
    // A lambda is named after the function that makes it: the program's own one that names it,
    // rather than a standard function given it to call.
    let mut maker: HashMap<&str, usize> = HashMap::new();
    for (index, body) in bodies.iter().enumerate() {
        for block in &body.blocks {
            let mut named: Vec<&str> = block
                .stmts
                .iter()
                .flat_map(|stmt| match &stmt.kind {
                    StmtKind::Assign { rvalue, .. } => rvalue_operands(rvalue),
                    _ => Vec::new(),
                })
                .chain(match &block.terminator {
                    Terminator::Call { args, .. } => args.iter().collect(),
                    _ => Vec::new(),
                })
                .filter_map(|operand| match operand {
                    Operand::Const(ember_mir::Const::Fn(symbol)) => Some(symbol.as_str()),
                    _ => None,
                })
                .collect();
            if let Terminator::Call { func: FuncRef::Direct { symbol, .. }, .. } = &block.terminator {
                named.push(symbol.as_str());
            }
            for symbol in named {
                let earlier = maker.entry(symbol).or_insert(index);
                if standard[*earlier] && !standard[index] {
                    *earlier = index;
                }
            }
        }
    }
    let name = |mut index: usize| {
        let mut name = String::new();
        while bodies[index].is_lambda && name.len() < 200 {
            name.push_str("lambda in ");
            match maker.get(bodies[index].symbol.as_str()) {
                Some(&outer) => index = outer,
                None => return format!("{name}a function"),
            }
        }
        name + &display_name(&bodies[index], types)
    };
    (0..bodies.len()).map(name).collect()
}

/// `[SEL-1]` (ODR-108) — one declaration of the program's own and the storage mechanism its declared
/// type uses: a row of IX.0's table, or `value`, `view`, `callable` or `raw pointer` for the types
/// the table has no row for.
#[derive(Clone, Debug)]
pub struct Declaration {
    /// `local`, `parameter`, `field` or `type`.
    pub kind: &'static str,
    /// The function a local or parameter belongs to; the type a field belongs to.
    pub owner: String,
    pub name: String,
    pub ty: String,
    pub mechanism: &'static str,
    pub span: Span,
}

/// `[SEL-1]` (ODR-108) — the storage mechanism `ty` uses: the row of IX.0's table its declared type
/// picks. A number, `bool`, `char`, `void` or range value is a `value` (its bytes, where it is); a
/// reference, span, `str` or guard is a `view` of another's storage; a callable value holds its
/// captured state (`callable`); a fixed array and a tuple are values like a struct.
pub fn storage_mechanism(types: &TypeTable, ty: Ty) -> &'static str {
    const AGGREGATE: &str = "struct, enum, tuple";
    match types.kind(ty) {
        TyKind::Bool | TyKind::Char | TyKind::Int(_) | TyKind::Uint(_) | TyKind::Float(_) | TyKind::Void | TyKind::Never | TyKind::Range(_) | TyKind::Const(_) => "value",
        TyKind::Str | TyKind::CStr | TyKind::Span { .. } | TyKind::Ref { .. } => "view",
        TyKind::Ptr { .. } | TyKind::Opaque(_) => "raw pointer",
        TyKind::Fn { .. } => "callable",
        TyKind::Vec { .. } => "Array, Map, Set, String",
        TyKind::Class(_) | TyKind::ClassInterface(_) => "class",
        TyKind::Tuple(_) | TyKind::Array { .. } | TyKind::ArrayN { .. } | TyKind::Enum(_) => AGGREGATE,
        TyKind::Dyn { .. } => "Box[T]",
        TyKind::Struct(_) => {
            let shown = types.display(ty);
            let head = shown.split('[').next().unwrap_or("");
            match head.rsplit('.').next().unwrap_or(head) {
                "Box" => "Box[T]",
                "Shared" => "Shared[T]",
                "SyncShared" => "SyncShared[T]",
                "Weak" => "Weak[O]",
                "Arena" | "FixedArena" | "ScopedArena" | "ArenaArray" | "ArenaMap" => "Arena, FixedArena",
                "Pool" | "Handle" => "Pool[T] + Handle[T]",
                "Cell" | "RefCell" | "OnceCell" => "Cell[T], RefCell[T]",
                "Mutex" | "RwLock" | "Atomic" | "Once" => "Mutex[T], RwLock[T], Atomic[T]",
                "Map" | "Set" | "SoA" | "String" | "Array" => "Array, Map, Set, String",
                "Ref" | "RefMut" | "MutexGuard" | "RwLockReadGuard" | "RwLockWriteGuard" | "SoARef" | "SoAMut" | "Span" | "MutSpan" => "view",
                _ => AGGREGATE,
            }
        }
        _ => AGGREGATE,
    }
}

/// `[SEL-1]` (ODR-108) — every declaration the program writes, with the mechanism it uses: each named
/// local and parameter of its functions, and each struct, enum and class it declares with their
/// fields. `standard` and `standard_file` say what is the standard library's, as for
/// `allocation_sites`.
pub fn declarations(
    bodies: &[Body],
    types: &TypeTable,
    standard: &[bool],
    standard_file: &dyn Fn(ember_span::FileId) -> bool,
) -> Vec<Declaration> {
    let own = |span: Span| span != Span::DUMMY && !standard_file(span.file);
    let names = function_names(bodies, types, standard);
    let mut out = Vec::new();
    for (_, def) in types.structs() {
        if !own(def.span) || def.origin.is_some() || def.name.as_str().contains('$') || def.name.as_str().starts_with("closure") {
            continue;
        }
        out.push(Declaration { kind: "type", owner: String::new(), name: def.name.to_string(), ty: "struct".into(), mechanism: "struct, enum, tuple", span: def.span });
        for field in &def.fields {
            out.push(Declaration { kind: "field", owner: def.name.to_string(), name: field.name.to_string(), ty: types.display(field.ty), mechanism: storage_mechanism(types, field.ty), span: field.span });
        }
    }
    for (_, def) in types.classes() {
        if !own(def.span) {
            continue;
        }
        out.push(Declaration { kind: "type", owner: String::new(), name: def.name.to_string(), ty: "class".into(), mechanism: "class", span: def.span });
        for field in &def.fields {
            out.push(Declaration { kind: "field", owner: def.name.to_string(), name: field.name.to_string(), ty: types.display(field.ty), mechanism: storage_mechanism(types, field.ty), span: field.span });
        }
    }
    for (_, def) in types.enums() {
        if !own(def.span) || def.origin.is_some() || def.name.as_str().starts_with("Option_") || def.name.as_str().starts_with("Result_") {
            continue;
        }
        out.push(Declaration { kind: "type", owner: String::new(), name: def.name.to_string(), ty: "enum".into(), mechanism: "struct, enum, tuple", span: def.span });
    }
    for (index, body) in bodies.iter().enumerate() {
        if standard[index] || !own(body.span) {
            continue;
        }
        for decl in &body.locals {
            let kind = match decl.kind {
                ember_mir::LocalKind::Arg => "parameter",
                ember_mir::LocalKind::User => "local",
                _ => continue,
            };
            let Some(name) = decl.name.as_deref() else { continue };
            // A lambda's environment is the compiler's, not a declaration of the program.
            let peeled = match types.kind(decl.ty) {
                TyKind::Ref { inner, .. } => *inner,
                _ => decl.ty,
            };
            if body.is_lambda && matches!(types.kind(peeled), TyKind::Struct(id) if types.struct_def(*id).name.as_str().starts_with("closure")) {
                continue;
            }
            if name.starts_with('$') || name.starts_with('_') && name.len() > 1 && name[1..].starts_with('_') || !own(decl.span) {
                continue;
            }
            out.push(Declaration { kind, owner: names[index].clone(), name: name.to_string(), ty: types.display(decl.ty), mechanism: storage_mechanism(types, decl.ty), span: decl.span });
        }
    }
    out.sort_by_key(|d| (d.span.file, d.span.start));
    out.dedup_by(|a, b| a.kind == b.kind && a.owner == b.owner && a.name == b.name && a.span == b.span);
    out
}

/// A method with the type it belongs to (`std.collections.Map.insert`), else the function's name.
fn method_name(body: &Body, types: &TypeTable) -> String {
    if body.class_owner.is_some() || body.name.contains('.') {
        return display_name(body, types);
    }
    let owner = body.args().next().filter(|(_, decl)| decl.name.as_deref() == Some("self")).and_then(|(_, decl)| {
        let ty = match types.kind(decl.ty) {
            TyKind::Ref { inner, .. } => *inner,
            _ => decl.ty,
        };
        match types.kind(ty) {
            TyKind::Struct(id) => {
                let def = types.struct_def(*id);
                Some(def.origin.as_ref().map_or(def.name.to_string(), |(origin, _)| origin.to_string()))
            }
            _ => None,
        }
    });
    match owner {
        Some(owner) => format!("{owner}.{}", body.name),
        None => body.name.clone(),
    }
}

/// The first place of the program's own that `body` reaches after statement `position` of
/// `block` (its terminator when `position` is past the statements).
fn own_place_after(body: &Body, block: usize, position: usize, own_file: &dyn Fn(Span) -> bool) -> Option<Span> {
    let spans = |block: &ember_mir::BasicBlock, from: usize| -> Vec<Span> {
        let statements = block.stmts.iter().skip(from).filter(|stmt| {
            !matches!(stmt.kind, StmtKind::StorageLive(_) | StmtKind::StorageDead(_) | StmtKind::Nop)
        });
        statements.map(|stmt| stmt.span).chain(std::iter::once(block.terminator_span)).collect()
    };
    let first = &body.blocks[block];
    if position < first.stmts.len()
        && let Some(span) = spans(first, position + 1).into_iter().find(|&span| own_file(span))
    {
        return Some(span);
    }
    let mut seen = HashSet::from([block]);
    let mut queue: VecDeque<usize> = crate::long_access_lint::successors(&first.terminator).into();
    while let Some(next) = queue.pop_front() {
        if !seen.insert(next) {
            continue;
        }
        let block = &body.blocks[next];
        if let Some(span) = spans(block, 0).into_iter().find(|&span| own_file(span)) {
            return Some(span);
        }
        queue.extend(crate::long_access_lint::successors(&block.terminator));
    }
    None
}

/// `[EFF-5]`, `[EFF-6]` — check every function's contracts against its effects. Returns how many
/// were broken.
pub fn check_effects_all(bodies: &[Body], types: &TypeTable, sink: &mut Sink) -> usize {
    if bodies.iter().all(|body| body.contracts.forbidden.is_empty()) {
        return 0;
    }
    let graph = Graph::new(bodies, types);
    let inherited = inherited_contracts(bodies, types);
    let mut reported = 0;
    for (index, body) in bodies.iter().enumerate() {
        let (from_base, base_name) = inherited[index].clone().unwrap_or((Effects::NONE, String::new()));
        let broken = (body.contracts.forbidden | from_base) & graph.effects[index];
        for (effect, _) in broken.each() {
            let Some((first, last, chain)) = graph.chain(index, effect, types) else { continue };
            // An allocation's failure is a panic too: under both contracts it is reported once,
            // as the allocation.
            if effect == Effects::PANIC && broken.contains(Effects::ALLOC) && last.effects.contains(Effects::ALLOC) {
                continue;
            }
            // A site inside a standard-library body the function was given is reported at the
            // function itself.
            let span = if first.span.file == body.span.file { first.span } else { body.span };
            let relaxed = matches!(last.origin, Origin::Operation(operation) if operation == RELAXED);
            let mut diagnostic = contract_diagnostic(body, effect, span, &chain, types, relaxed);
            if from_base.contains(effect) && !body.contracts.forbidden.contains(effect) {
                diagnostic = diagnostic.note(format!("the contract is `{base_name}`'s: an override inherits its base method's contracts (EFF-8)"));
            }
            sink.emit(diagnostic);
            reported += 1;
        }
    }
    reported
}

fn contract_diagnostic(
    body: &Body,
    effect: Effects,
    span: Span,
    chain: &[String],
    types: &TypeTable,
    relaxed: bool,
) -> Diagnostic {
    let (code, contract, what) = match effect {
        Effects::NONDET if relaxed => {
            (codes::E4072, "@deterministic", "reaches relaxed floating point (`@fastmath`, `@fp(contract)`)")
        }
        Effects::ALLOC => (codes::E4001, "@noalloc", "reaches an allocation"),
        Effects::SYNC => (codes::E4002, "@nosync", "reaches a synchronising operation"),
        Effects::BLOCK => (codes::E4003, "@noblock", "reaches an operation that may wait"),
        Effects::IO => (codes::E4041, "@noio", "reaches I/O"),
        Effects::LOCK => (codes::E4042, "@nolock", "acquires a lock"),
        Effects::PANIC => (codes::E4040, "@nopanic(explicit)", "reaches a panic"),
        _ => (codes::E4070, "@deterministic", "reaches a nondeterministic operation"),
    };
    let realtime = body.contracts.realtime.contains(effect);
    let named = if realtime { format!("`@realtime` (`{contract}`)") } else { format!("`{contract}`") };
    let mut diagnostic = Diagnostic::error(code, span, format!("{named} function `{}` {what}", display_name(body, types)))
        .primary_label("reached from here")
        .note(chain.join(" -> "));
    if effect == Effects::PANIC {
        diagnostic = diagnostic.note(
            "`@nopanic(explicit)` forbids only the panics the program writes and allocation failure; \
             arithmetic, bounds, aliasing and stale-handle checks are allowed under it (EFF-16)",
        );
    }
    if realtime {
        diagnostic = diagnostic.note(format!(
            "`@realtime` stands for `{}` here, the set its package's manifest declares (by default `@noalloc @nolock @noblock @nopanic(explicit)`), and this is `{contract}` (EFF-19)",
            body.contracts.realtime.contract_names()
        ));
    }
    diagnostic
}

/// `[EFF-8]` — an override inherits its base method's contracts: for each method, the contracts
/// of the methods in the same slot of the classes above its own, and the nearest one's name.
fn inherited_contracts(bodies: &[Body], types: &TypeTable) -> Vec<Option<(Effects, String)>> {
    let ancestors = class_ancestors(types);
    bodies
        .iter()
        .map(|body| {
            let (Some(owner), Some(slot)) = (body.class_owner, body.class_virtual_slot) else { return None };
            let chain = ancestors.get(&owner)?;
            let mut forbidden = Effects::NONE;
            let mut name = None;
            for base in chain.iter().skip(1) {
                for candidate in bodies {
                    if candidate.class_owner == Some(*base)
                        && candidate.class_virtual_slot == Some(slot)
                        && !candidate.contracts.forbidden.is_empty()
                    {
                        forbidden |= candidate.contracts.forbidden;
                        name.get_or_insert_with(|| display_name(candidate, types));
                    }
                }
            }
            name.map(|name| (forbidden, name))
        })
        .collect()
}

/// A function as the program names it: a class method with its class.
fn display_name(body: &Body, types: &TypeTable) -> String {
    match body.class_owner {
        Some(class) if !body.name.contains('.') => format!("{}.{}", types.class_def(class).name, body.name),
        _ => body.name.clone(),
    }
}

struct Graph<'a> {
    bodies: &'a [Body],
    sites: Vec<Vec<Site>>,
    effects: Vec<Effects>,
}

impl<'a> Graph<'a> {
    fn new(bodies: &'a [Body], types: &TypeTable) -> Graph<'a> {
        let index: HashMap<&str, usize> =
            bodies.iter().enumerate().map(|(i, body)| (body.symbol.as_str(), i)).collect();
        let mut builder = SiteBuilder {
            bodies,
            types,
            index: &index,
            ancestors: class_ancestors(types),
            drops: HashMap::new(),
        };
        let sites: Vec<Vec<Site>> = bodies.iter().map(|body| builder.sites(body)).collect();
        let effects = fixed_point(&sites);
        Graph { bodies, sites, effects }
    }

    /// `[EFF-6]` — the nearest operation that gives `from` the effect: the site in `from` it is
    /// reached through, and the chain of names down to the operation.
    fn chain(&self, from: usize, effect: Effects, types: &TypeTable) -> Option<(Site, Site, Vec<String>)> {
        let mut parent: HashMap<usize, (usize, Site)> = HashMap::new();
        let mut seen = HashSet::from([from]);
        let mut queue = VecDeque::from([from]);
        while let Some(at) = queue.pop_front() {
            for site in &self.sites[at] {
                if !site.effects.intersects(effect) {
                    continue;
                }
                match site.origin {
                    Origin::Operation(operation) => {
                        let mut names = vec![operation.to_string()];
                        let mut current = at;
                        let mut first = *site;
                        while let Some(&(caller, call)) = parent.get(&current) {
                            names.push(display_name(&self.bodies[current], types));
                            first = call;
                            current = caller;
                        }
                        names.push(display_name(&self.bodies[from], types));
                        names.reverse();
                        return Some((first, *site, names));
                    }
                    Origin::Call(callee) => {
                        if self.effects[callee].contains(effect) && seen.insert(callee) {
                            parent.insert(callee, (at, *site));
                            queue.push_back(callee);
                        }
                    }
                }
            }
        }
        None
    }
}

/// Each body's effects: its own operations' and, until nothing changes, its callees'.
fn fixed_point(sites: &[Vec<Site>]) -> Vec<Effects> {
    let mut effects: Vec<Effects> = sites
        .iter()
        .map(|own| own.iter().filter(|site| matches!(site.origin, Origin::Operation(_))).fold(Effects::NONE, |all, site| all | site.effects))
        .collect();
    let mut callers: Vec<Vec<usize>> = vec![Vec::new(); sites.len()];
    for (caller, own) in sites.iter().enumerate() {
        for site in own {
            if let Origin::Call(callee) = site.origin {
                callers[callee].push(caller);
            }
        }
    }
    let mut queue: VecDeque<usize> = (0..sites.len()).collect();
    let mut queued = vec![true; sites.len()];
    while let Some(callee) = queue.pop_front() {
        queued[callee] = false;
        for &caller in &callers[callee] {
            let joined = effects[caller] | effects[callee];
            if joined != effects[caller] {
                effects[caller] = joined;
                if !queued[caller] {
                    queued[caller] = true;
                    queue.push_back(caller);
                }
            }
        }
    }
    effects
}

/// Each class's ancestors, itself included.
fn class_ancestors(types: &TypeTable) -> HashMap<ClassId, Vec<ClassId>> {
    let mut out = HashMap::new();
    for (id, _) in types.runtime_classes() {
        let mut chain = Vec::new();
        let mut current = Some(id);
        while let Some(class) = current {
            chain.push(class);
            current = types.class_def(class).base;
        }
        out.insert(id, chain);
    }
    out
}

struct SiteBuilder<'a> {
    bodies: &'a [Body],
    types: &'a TypeTable,
    index: &'a HashMap<&'a str, usize>,
    ancestors: HashMap<ClassId, Vec<ClassId>>,
    /// What dropping a value of each type runs: its own effects and the `drop` bodies it calls.
    drops: HashMap<Ty, (Effects, Vec<usize>)>,
}

impl SiteBuilder<'_> {
    fn sites(&mut self, body: &Body) -> Vec<Site> {
        let mut sites = Vec::new();
        if body.is_unsafe {
            own(Effects::UNSAFE, body.span, "an `unsafe fn`", &mut sites);
        }
        // `[EFF-3]`, `[DET-2]` — a foreign function carries `FFI`, and is `Nondet` unless its
        // declaration says `@ffi(deterministic)`, which is not built yet.
        if body.is_extern_declaration || (body.abi.is_some() && body.blocks.is_empty()) {
            own(Effects::FFI | Effects::NONDET, body.span, "a foreign function", &mut sites);
        }
        // `[DET-2]` — floating-point contraction and every `@fastmath` relaxation.
        if body.fp != ember_types::FpMode::Strict {
            own(Effects::NONDET, body.span, RELAXED, &mut sites);
        }
        for block in &body.blocks {
            for stmt in &block.stmts {
                self.stmt_sites(body, stmt, &mut sites);
            }
            self.terminator_sites(body, block, &mut sites);
        }
        sites
    }

    fn stmt_sites(&mut self, body: &Body, stmt: &ember_mir::Stmt, sites: &mut Vec<Site>) {
        match &stmt.kind {
            // `[EFF-9]` — a class exclusivity check left after the proofs.
            StmtKind::BeginAccess { .. } | StmtKind::BeginAccessTransfer { .. } => {
                own(Effects::CHECK_ALIASING, stmt.span, "a class exclusivity check", sites);
            }
            StmtKind::Drop { place, .. } => {
                let ty = place_type(body, self.types, place);
                self.drop_sites(ty, stmt.span, sites);
            }
            StmtKind::Assign { rvalue, .. } => {
                for operand in rvalue_operands(rvalue) {
                    self.sync_copy(body, operand, stmt.span, sites);
                }
                // `[CLO-10]` — captured state larger than three pointer-sized words goes to
                // the heap (the backend's own test is the same: D-620).
                if let Rvalue::Aggregate { kind: AggregateKind::Callable, operands } = rvalue
                    && let Some(Operand::Copy(env) | Operand::Move(env)) = operands.get(1)
                    && self.types.layout(place_type(body, self.types, env)).size > 3 * self.types.pointer_size()
                {
                    own(
                        Effects::ALLOC | Effects::PANIC,
                        stmt.span,
                        "a callable value whose captured state is larger than three words",
                        sites,
                    );
                }
            }
            _ => {}
        }
    }

    fn terminator_sites(&mut self, body: &Body, block: &ember_mir::BasicBlock, sites: &mut Vec<Site>) {
        match &block.terminator {
            Terminator::Call { func, args, .. } => {
                let span = call_span(block);
                for operand in args {
                    self.sync_copy(body, operand, span, sites);
                }
                self.call_sites(func, span, sites);
                // `[PAR-4]` — a `@parallel` loop runs its chunk closure and its combiner here.
                if let FuncRef::Builtin { which: Builtin::ParallelFor { .. }, .. } = func {
                    for operand in args {
                        if let Operand::Const(ember_mir::Const::Fn(symbol)) = operand
                            && let Some(&callee) = self.index.get(symbol.as_str())
                        {
                            sites.push(Site { effects: Effects::ALL, span, origin: Origin::Call(callee) });
                        }
                    }
                }
                // `[JOB-1]` — a job's state that may not fit its slot's 64 bytes is one allocation.
                if let FuncRef::Builtin { which: Builtin::JobSubmit { closure: Some(_) } | Builtin::JobScopeSubmit { closure: Some(_), .. }, .. } = func
                    && let Some(Operand::Copy(place) | Operand::Move(place)) = args.last()
                    && !job_state_fits(self.types, place_type(body, self.types, place))
                {
                    sites.push(Site { effects: Effects::ALLOC, span, origin: Origin::Operation("a job's state, larger than its slot") });
                }
            }
            Terminator::Assert { msg, span, .. } => {
                let (effects, operation) = match msg {
                    AssertKind::Overflow(_)
                    | AssertKind::DivisionByZero
                    | AssertKind::SignedDivisionOverflow
                    | AssertKind::ShiftTooLarge => (Effects::CHECK_ARITHMETIC, "an arithmetic check"),
                    AssertKind::Bounds { .. } => (Effects::CHECK_BOUNDS, "a bounds check"),
                    AssertKind::RefCellBorrow { .. } => (Effects::CHECK_ALIASING, "a `RefCell` borrow check"),
                    AssertKind::Downcast => (Effects::PANIC, "`as!`"),
                    AssertKind::Panic { .. } => (Effects::PANIC, "a panic"),
                };
                own(effects, *span, operation, sites);
            }
            _ => {}
        }
    }

    /// `[EFF-1]`'s `Sync`: retaining a `Sync` handle is an atomic operation.
    fn sync_copy(&self, body: &Body, operand: &Operand, span: Span, sites: &mut Vec<Site>) {
        let Operand::Copy(place) = operand else { return };
        if body.uncounted_handles.contains(&place.local) && place.projection.is_empty() {
            return;
        }
        if let TyKind::Class(id) = self.types.kind(place_type(body, self.types, place))
            && self.types.class_def(*id).is_sync
        {
            sites.push(Site { effects: Effects::SYNC, span, origin: Origin::Operation("retaining a `@sync` handle") });
        }
    }

    fn call_sites(&mut self, func: &FuncRef, span: Span, sites: &mut Vec<Site>) {
        match func {
            FuncRef::Direct { symbol, .. } => {
                if let Some(&callee) = self.index.get(symbol.as_str()) {
                    sites.push(Site { effects: Effects::ALL, span, origin: Origin::Call(callee) });
                }
            }
            FuncRef::Virtual { slot, receiver, .. } => {
                // `[DSP-1]` — the object is of `receiver`'s class or one below it: the body in
                // the slot is one of theirs, or one they inherit.
                let related = |owner: ClassId| {
                    self.ancestors.get(receiver).is_some_and(|chain| chain.contains(&owner))
                        || self.ancestors.get(&owner).is_some_and(|chain| chain.contains(receiver))
                };
                for (callee, candidate) in self.bodies.iter().enumerate() {
                    if candidate.class_virtual_slot == Some(*slot)
                        && candidate.class_owner.is_some_and(related)
                        && !candidate.is_abstract
                    {
                        sites.push(Site { effects: Effects::ALL, span, origin: Origin::Call(callee) });
                    }
                }
            }
            FuncRef::Interface { .. } => sites.push(Site {
                effects: Effects::ALL,
                span,
                origin: Origin::Operation("a call through a `dyn` value, whose effects nothing declares"),
            }),
            FuncRef::Indirect { .. } => sites.push(Site {
                effects: Effects::ALL,
                span,
                origin: Origin::Operation("a call through a function value, whose effects nothing declares"),
            }),
            FuncRef::DynBoxNew { .. } => sites.push(Site {
                effects: Effects::ALLOC | Effects::PANIC,
                span,
                origin: Origin::Operation("`Box[dyn …]`"),
            }),
            FuncRef::Builtin { which, arg_ty } => {
                let (effects, operation) = builtin_effects(which);
                if !effects.is_empty() {
                    sites.push(Site { effects, span, origin: Origin::Operation(operation) });
                }
                match which {
                    // The elements a call removes are dropped.
                    Builtin::ArrayClear | Builtin::ArrayTruncate => {
                        if let Some(elem) = self.element(*arg_ty) {
                            self.drop_sites(elem, span, sites);
                        }
                    }
                    Builtin::MemDrop { elem } => self.drop_sites(*elem, span, sites),
                    // ADR-186 — formatting a value runs the written `Debug`
                    // of each part of it that has one.
                    Builtin::Format | Builtin::FormatWith(_) | Builtin::Print | Builtin::EPrint => {
                        for writer in self.types.debug_writers_reached(*arg_ty) {
                            if let Some(&callee) = self.index.get(writer) {
                                sites.push(Site { effects: Effects::ALL, span, origin: Origin::Call(callee) });
                            }
                        }
                    }
                    // `[STA-3]` — the first access runs the initialiser.
                    Builtin::LazyStatic { init, .. } => {
                        if let Some(&callee) = self.index.get(init.as_str()) {
                            sites.push(Site { effects: Effects::ALL, span, origin: Origin::Call(callee) });
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    fn element(&self, ty: Ty) -> Option<Ty> {
        let ty = match self.types.kind(ty) {
            TyKind::Ref { inner, .. } => *inner,
            _ => ty,
        };
        match self.types.kind(ty) {
            TyKind::Vec { elem, .. } | TyKind::Span { elem, .. } => Some(*elem),
            _ => None,
        }
    }

    fn drop_sites(&mut self, ty: Ty, span: Span, sites: &mut Vec<Site>) {
        let (effects, callees) = self.drop_of(ty);
        if !effects.is_empty() {
            sites.push(Site { effects, span, origin: Origin::Operation(drop_description(effects)) });
        }
        for callee in callees {
            sites.push(Site { effects: Effects::ALL, span, origin: Origin::Call(callee) });
        }
    }

    /// What dropping a value of `ty` runs (`[OWN-2]`, `[DRP-6]`): each `drop` method it reaches,
    /// and `Sync` for the release of a `Sync` handle.
    fn drop_of(&mut self, ty: Ty) -> (Effects, Vec<usize>) {
        if let Some(known) = self.drops.get(&ty) {
            return known.clone();
        }
        self.drops.insert(ty, (Effects::NONE, Vec::new()));
        let mut effects = Effects::NONE;
        let mut callees = Vec::new();
        let mut parts: Vec<Ty> = Vec::new();
        match self.types.kind(ty).clone() {
            // `[THR-5]`, `[JOB-2]` — a `Scope`'s end joins its threads, a `JobScope`'s waits for its jobs.
            TyKind::Struct(id)
                if self.types.struct_def(id).origin.as_ref().is_some_and(|(origin, _)| matches!(origin.as_str(), "std.thread.Scope" | "std.jobs.JobScope")) =>
            {
                effects |= Effects::SYNC | Effects::BLOCK;
            }
            // `[THR-3]` — a `MutexGuard`'s end releases its lock: a synchronising operation.
            TyKind::Struct(id)
                if self.types.struct_def(id).origin.as_ref().is_some_and(|(origin, _)| {
                    matches!(
                        origin.as_str(),
                        "std.sync.MutexGuard" | "std.sync.RwLockReadGuard" | "std.sync.RwLockWriteGuard" | "std.thread.JoinHandle"
                    )
                }) =>
            {
                effects |= Effects::SYNC;
            }
            TyKind::Struct(id) => {
                let def = self.types.struct_def(id);
                if def.has_drop {
                    self.push_drop_body(&self.types.symbol_name(ty), &mut callees);
                }
                parts.extend(def.fields.iter().map(|field| field.ty));
                if let Some((_, args)) = &def.origin {
                    parts.extend(args.iter().copied());
                }
            }
            TyKind::Enum(id) => {
                let def = self.types.enum_def(id);
                if def.has_drop {
                    self.push_drop_body(&self.types.symbol_name(ty), &mut callees);
                }
                parts.extend(def.variants.iter().flat_map(|variant| variant.fields.iter().map(|field| field.ty)));
            }
            TyKind::Tuple(items) => parts.extend(items),
            TyKind::Array { elem, .. } | TyKind::ArrayN { elem, .. } | TyKind::Vec { elem, .. } => parts.push(elem),
            TyKind::Class(id) => {
                // The last handle's release drops the object, whose class may be any below `id`.
                for (class, chain) in &self.ancestors {
                    if !chain.contains(&id) {
                        continue;
                    }
                    let def = self.types.class_def(*class);
                    if def.is_sync {
                        effects |= Effects::SYNC;
                    }
                    for link in chain {
                        let link_def = self.types.class_def(*link);
                        if link_def.has_drop {
                            let symbol = ember_branding::drop_function(&link_def.name.to_string());
                            if let Some(&callee) = self.index.get(symbol.as_str()) {
                                callees.push(callee);
                            }
                        }
                        parts.extend(link_def.fields.iter().map(|field| field.ty));
                    }
                }
            }
            // A value that may own anything: its drop's effects are not known here.
            TyKind::Dyn { .. } | TyKind::ClassInterface(_) => effects |= Effects::ALL,
            TyKind::Fn { stateless: false, .. } => effects |= Effects::ALL,
            _ => {}
        }
        for part in parts {
            if part != ty && self.types.needs_drop(part) {
                let (more, bodies) = self.drop_of(part);
                effects |= more;
                callees.extend(bodies);
            }
        }
        callees.sort_unstable();
        callees.dedup();
        self.drops.insert(ty, (effects, callees.clone()));
        (effects, callees)
    }

    fn push_drop_body(&self, owner: &str, callees: &mut Vec<usize>) {
        let symbol = ember_branding::drop_function(owner);
        if let Some(&callee) = self.index.get(symbol.as_str()) {
            callees.push(callee);
        }
    }
}

/// An operation of the body's own with these effects.
fn own(effects: Effects, span: Span, operation: &'static str, sites: &mut Vec<Site>) {
    if !effects.is_empty() {
        sites.push(Site { effects, span, origin: Origin::Operation(operation) });
    }
}

/// `[DET-5]` — a relaxed function reached from a `@deterministic` one is `E4072`.
const RELAXED: &str = "a function with `@fastmath` or `@fp(contract)`";

fn drop_description(effects: Effects) -> &'static str {
    if effects == Effects::SYNC {
        "releasing a `@sync` handle or a lock"
    } else {
        "dropping a value whose drop is not known here"
    }
}

fn rvalue_operands(rvalue: &Rvalue) -> Vec<&Operand> {
    match rvalue {
        Rvalue::Use(operand) | Rvalue::UnaryOp { operand, .. } | Rvalue::Cast { operand, .. } => vec![operand],
        Rvalue::BinaryOp { lhs, rhs, .. } => vec![lhs, rhs],
        Rvalue::Aggregate { operands, .. } => operands.iter().collect(),
        Rvalue::Repeat { value, .. } => vec![value],
        Rvalue::Discriminant(_) | Rvalue::Ref { .. } => Vec::new(),
    }
}

fn call_span(block: &ember_mir::BasicBlock) -> Span {
    block.terminator_span
}

/// `[EFF-1]`'s table, for each operation the compiler provides. An allocation also carries
/// `Panic(Explicit)`: allocation failure panics.
fn builtin_effects(which: &Builtin) -> (Effects, &'static str) {
    const ALLOC: Effects = Effects(Effects::ALLOC.0 | Effects::PANIC.0);
    use Builtin as B;
    match which {
        B::Println | B::EPrintln | B::EPrint | B::Print => (Effects::IO, "console output"),
        B::Input => (Effects::IO | Effects::BLOCK | ALLOC, "`input`, which reads the console"),
        B::Panic => (Effects::PANIC, "`panic`"),
        B::Assert => (Effects::PANIC, "`assert`"),
        B::ClassDowncast { forced: true, .. } => (Effects::PANIC, "`as!`"),
        B::ClassDowncast { .. } => (Effects::NONE, ""),
        B::RangeChecked(_) => (Effects::PANIC, "a checked range conversion"),
        B::FixedArenaAlloc { .. } => (Effects::PANIC, "`FixedArena.alloc`, which panics when full"),
        B::ArrayNew | B::ArrayFromLiteral => (ALLOC, "a new `Array`"),
        B::StringNew => (ALLOC, "a new `String`"),
        B::ClassNew { .. } => (ALLOC, "a new class instance"),
        B::BoxNew { .. } => (ALLOC, "`Box(…)`"),
        B::SharedNew { .. } => (ALLOC, "`Shared(…)`"),
        B::ArrayPush => (ALLOC, "`Array.push`"),
        B::ArrayInsert => (ALLOC | Effects::CHECK_BOUNDS, "`Array.insert`"),
        B::ArrayExtend => (ALLOC, "`Array.extend`"),
        B::ArrayReserve | B::ArrayReserveHint { .. } => (ALLOC, "`Array.reserve`"),
        B::ArrayClone { .. } | B::CloneParts { .. } => (ALLOC, "a clone"),
        B::ArraySorted { .. } => (ALLOC, "`sorted`"),
        B::ArrayDrain => (ALLOC | Effects::CHECK_BOUNDS, "`Array.drain`"),
        B::StringPush | B::StringPushChar => (ALLOC, "`String.push`"),
        B::StringInsert => (ALLOC | Effects::CHECK_BOUNDS, "`String.insert`"),
        B::Format | B::FormatWith(_) => (ALLOC, "an f-string"),
        B::StrReplace => (ALLOC, "`str.replace`"),
        B::StrRepeat => (ALLOC, "`str.repeat`"),
        B::StrToUpper | B::StrToLower => (ALLOC, "a case conversion"),
        B::ProcessArgs => (ALLOC, "`args`"),
        B::MemAlloc => (ALLOC | Effects::UNSAFE, "`mem.alloc`"),
        B::ArenaWithCapacity | B::ArenaArrayWithCapacity { .. } | B::ArenaMapWithCapacity { .. } => {
            (ALLOC, "a new arena")
        }
        // `[ARN-4]` — a growing `Arena` carries `Alloc` on its growth path.
        B::ArenaAlloc { .. }
        | B::ArenaAllocUninit { .. }
        | B::ArenaAllocArrayZeroed { .. }
        | B::ArenaAllocArrayDefault { .. }
        | B::ScopedArenaAlloc { .. } => (ALLOC, "an arena allocation, which may grow the arena"),
        B::ArrayRemove | B::ArraySwap | B::ArraySwapRemove | B::Slice { .. } | B::StringRemove
        | B::StringTruncate | B::RangeNth | B::SpanSplitAt { .. } | B::SpanChunksNew { .. }
        | B::SpanWindowsNew { .. } | B::ArenaArrayInsert { .. } | B::ArenaArrayRemove { .. } => {
            (Effects::CHECK_BOUNDS, "a bounds check")
        }
        B::RefCellBorrow | B::RefCellBorrowMut | B::RefCellTryBorrow | B::RefCellTryBorrowMut => {
            (Effects::CHECK_ALIASING, "a `RefCell` borrow check")
        }
        // `[EFF-18]` — a blocking lock is `Sync + Lock + Block`; `try_lock` is `Sync + Lock`.
        B::MutexLock => (Effects::SYNC | Effects::LOCK | Effects::BLOCK, "`Mutex.lock`"),
        B::MutexTryLock => (Effects::SYNC | Effects::LOCK, "`Mutex.try_lock`"),
        B::RwLockAcquire { .. } => (Effects::SYNC | Effects::LOCK | Effects::BLOCK, "`RwLock.read`/`write`"),
        B::RwLockTryAcquire { .. } => (Effects::SYNC | Effects::LOCK, "`RwLock.try_read`/`try_write`"),
        B::Atomic { op: ember_mir::AtomicOp::Wait, .. } => (Effects::SYNC | Effects::BLOCK, "a wait on an `Atomic`"),
        B::Atomic { .. } => (Effects::SYNC, "an `Atomic` operation"),
        B::ThreadSpawn => (Effects::SYNC | Effects::ALLOC, "`thread.spawn`"),
        B::ThreadJoin => (Effects::SYNC | Effects::BLOCK, "`JoinHandle.join`"),
        B::ThreadScope => (Effects::ALLOC, "`thread.scope`"),
        B::SoA { op: ember_mir::SoAOp::WithCapacity | ember_mir::SoAOp::Push | ember_mir::SoAOp::Reserve, .. } => {
            (Effects::ALLOC, "growing an `SoA`")
        }
        B::SoA { op: ember_mir::SoAOp::SwapRemove, .. } => (Effects::PANIC, "`SoA.swap_remove`"),
        B::SoA { op: ember_mir::SoAOp::Truncate | ember_mir::SoAOp::DropAt, .. } => (Effects::NONE, "dropping an `SoA`'s elements"),
        B::SoA { .. } => (Effects::NONE, "an `SoA` operation"),
        B::ScopeSpawn { .. } => (Effects::SYNC | Effects::ALLOC, "`Scope.spawn`"),
        B::ScopedJoin => (Effects::SYNC | Effects::BLOCK, "`ScopedJoinHandle.join`"),
        // `[JOB-1]`, `[JOB-2]` — a job is handed to the pool; one with more than six jobs to wait
        // for first waits for all but six of them to end. A state larger than its slot is one
        // allocation (`call` sites, below).
        B::JobScope => (Effects::NONE, "`jobs.scope`"),
        B::JobScopeSubmit { after: true, .. } => (Effects::SYNC | Effects::BLOCK, "`JobScope.submit_after`"),
        B::JobScopeSubmit { .. } => (Effects::SYNC, "`JobScope.submit`"),
        B::JobSubmit { .. } => (Effects::SYNC, "`jobs.submit`"),
        B::JobWait => (Effects::SYNC | Effects::BLOCK, "`jobs.wait`"),
        // `[PAR-4]` — a `@parallel` loop waits for its chunks, and allocates nothing.
        B::ParallelFor { .. } => (Effects::SYNC | Effects::BLOCK, "a `@parallel` loop"),
        // `[DSJ-1]` — it establishes a static fact; `@static_safe` allows it (`[EFF-12]`).
        B::AssertDisjoint { .. } => (Effects::CHECK_ALIASING, "`assert_disjoint`"),
        B::WeakUpgrade { .. } => (Effects::CHECK_STALE, "`Weak.upgrade`"),
        B::LazyStatic { .. } => (Effects::SYNC | Effects::BLOCK, "a static's first access, which may wait for another thread's"),
        B::ConstStatic { .. } => (Effects::NONE, "a static in the image"),
        // `[CT-2]` — the two inputs compile time is given, recorded as build inputs.
        B::ComptimeReadFile | B::ComptimeEnv { .. } => (Effects::NONE, "a build input"),
        B::PtrRead | B::PtrWrite | B::MemFree | B::SpanGetUnchecked | B::RangeNewUnchecked(_)
        | B::MaybeUninitAssumeInit { .. } | B::MaybeUninitSpanAssumeInit { .. } | B::MaybeUninitWriteAt { .. } => {
            (Effects::UNSAFE, "an `unsafe` operation")
        }
        B::ForeignStaticRead { .. } | B::ForeignStaticWrite { .. } | B::ForeignStaticFieldWrite { .. } => {
            (Effects::FFI | Effects::UNSAFE, "a foreign static")
        }
        // `[DET-2]` — a transcendental function from the platform library.
        B::FloatPow => (Effects::NONDET, "the platform's `pow`"),
        B::FloatLib(lib) => match lib {
            FloatLib::Sqrt | FloatLib::Floor | FloatLib::Ceil | FloatLib::Trunc | FloatLib::Round
            | FloatLib::RoundHalfAway | FloatLib::Copysign | FloatLib::MulAdd => (Effects::NONE, ""),
            _ => (Effects::NONDET, "a transcendental function from the platform library"),
        },
        // `[DET-2]` — the system random source (`RandomState`'s seed).
        B::ProcessKey => (Effects::NONDET, "the process's random hash key"),
        B::ViewsOverlap { .. } | B::TotalLess | B::FloatAbs | B::IntOverflowing(_) | B::IntBits(_)
        | B::RangeCount | B::StrCharCount | B::StrStartsWith | B::StrEndsWith | B::StrFind { .. }
        | B::StrSplitOnce { .. } | B::StrCount | B::StrTrimStart | B::StrTrimEnd | B::StrSliceOk
        | B::Utf8Valid | B::SpanToStr | B::CStrToSpan | B::CStringAsCStr | B::ParseStatus { .. }
        | B::ParseValue { .. } | B::StrCharNext | B::StrCharNextWide | B::StrContains | B::StrContainsChar
        | B::ArraySort | B::ArrayReverse | B::ArrayClear | B::ArrayPop { .. } | B::ArrayCapacity
        | B::ArrayTruncate | B::ValueCompare { .. } | B::ClassSuperInit { .. } | B::WeakNew { .. }
        | B::WeakEmpty { .. } | B::StrIsCharBoundary | B::StrAsBytes | B::ArrayLen | B::StringLen
        | B::StringAsStr | B::PtrIsNull | B::SizeOf | B::AlignOf | B::MemReplace { .. } | B::MemTake { .. }
        | B::MemSwap { .. } | B::MemForget { .. } | B::MemDrop { .. } | B::SpanLen | B::SpanGet
        | B::SpanFrom { .. } | B::SpanReborrow | B::SpanSharedReborrow | B::SpanIterNext { .. }
        | B::SpanIterNextBack { .. } | B::SpanIterLen | B::SpanChunksNext { .. } | B::SpanWindowsNext { .. }
        | B::SpanAsPtr { .. } | B::RangeContains(_) | B::RangeClamped(_) | B::CellSet | B::CellReplace
        | B::CellIntoInner | B::CellUpdate | B::CellUpdateDefault { .. } | B::CellTake { .. }
        | B::MaybeUninitUninit { .. } | B::MaybeUninitWrite { .. } | B::UnsafeCellGet { .. }
        | B::UnsafeCellIntoInner | B::ArenaReset | B::FixedArenaReset | B::ArenaScope { .. }
        | B::ArenaArrayGet { .. } | B::ArenaArrayPush { .. } | B::ArenaArrayClear | B::ArenaArrayIterNext { .. }
        | B::ArenaMapGet { .. } | B::ArenaMapContains { .. } | B::ArenaMapInsert { .. } | B::ArenaMapRemove { .. }
        | B::ArenaMapClear | B::ArenaMapIterNext { .. } => (Effects::NONE, ""),
    }
}
