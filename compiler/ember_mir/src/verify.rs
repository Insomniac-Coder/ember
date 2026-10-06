//! The MIR verifier (Part XVIII §4.2).
//!
//! Runs after transformations in debug builds and unconditionally at the
//! code-generation boundary. The base verifier checks structural invariants —
//! well-formedness of the CFG and of places — while the typed view verifier
//! checks the provenance-carrying shapes code generation is allowed to see.
//! `[MIR-4]`: every block ends with exactly one terminator, which the type
//! system already guarantees, so the check here is that every terminator names
//! a block that exists.
//!
//! The invariants that need type information (`[MIR-1]` well-typed places,
//! `[MIR-2]` at most one move per path, `[MIR-3]` access brackets,
//! `[MIR-5]` retain/release only on handles) are added by the phases that
//! introduce the constructs they govern.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::{
    BasicBlockId, Body, Builtin, Const, FuncRef, LocalId, LocalKind, Operand, Place, Projection, Rvalue,
    StmtKind, Terminator,
};
use ember_types::{Ty, TyKind, TypeTable};

#[derive(Debug)]
pub struct Violation {
    pub body: String,
    pub message: String,
}

/// A MIR program that has passed every verifier required by the current C
/// backend boundary. The constructor is private to this module, so a backend
/// cannot accidentally consume a raw `&[Body]` after a transformation.
pub struct VerifiedMir<'a> {
    bodies: &'a [Body],
    types: &'a TypeTable,
}

impl VerifiedMir<'_> {
    pub fn bodies(&self) -> &[Body] {
        self.bodies
    }

    pub fn types(&self) -> &TypeTable {
        self.types
    }
}

/// Accumulates violations while walking one body.
struct Verifier<'a> {
    symbol: &'a str,
    local_count: u32,
    block_count: u32,
    violations: Vec<Violation>,
}

impl Verifier<'_> {
    fn fail(&mut self, message: String) {
        self.violations.push(Violation {
            body: self.symbol.to_string(),
            message,
        });
    }

    fn local(&mut self, id: LocalId, at: &str) {
        if id.0 >= self.local_count {
            self.fail(format!("{at} names _{}, which does not exist", id.0));
        }
    }

    fn place(&mut self, place: &Place, at: &str) {
        self.local(place.local, at);
        for projection in &place.projection {
            if let crate::Projection::Index(local) = projection {
                self.local(*local, at);
            }
        }
    }

    fn operand(&mut self, operand: &Operand, at: &str) {
        match operand {
            Operand::Copy(p) | Operand::Move(p) => self.place(p, at),
            Operand::Const(_) => {}
        }
    }

    fn rvalue(&mut self, rvalue: &Rvalue, at: &str) {
        match rvalue {
            Rvalue::Use(o) | Rvalue::UnaryOp { operand: o, .. } => self.operand(o, at),
            Rvalue::Cast { operand, .. } => self.operand(operand, at),
            Rvalue::BinaryOp { lhs, rhs, .. } => {
                self.operand(lhs, at);
                self.operand(rhs, at);
            }
            Rvalue::Aggregate { operands, .. } => {
                for o in operands {
                    self.operand(o, at);
                }
            }
            Rvalue::Repeat { value, .. } => self.operand(value, at),
            Rvalue::Discriminant(place) => self.place(place, at),
            Rvalue::Ref { place, .. } => self.place(place, at),
        }
    }

    fn target(&mut self, target: BasicBlockId, at: &str) {
        if target.0 >= self.block_count {
            self.fail(format!(
                "{at} jumps to bb{}, which does not exist",
                target.0
            ));
        }
    }

    /// A dynamic call carries the complete declaration-derived table layout.
    /// Check the selected signature here rather than letting a malformed MIR
    /// body reach C emission, where a mismatched function-pointer cast would
    /// be undefined behaviour rather than a recoverable compiler failure.
    fn interface_call(&mut self, func: &FuncRef, at: &str) {
        let FuncRef::Interface { slot, params, param_modes, ret, layout, .. } = func else {
            return;
        };
        let Some(signature) = layout.get(*slot).and_then(|signature| signature.as_ref()) else {
            self.fail(format!("{at}: dynamic interface call targets non-callable slot {slot}"));
            return;
        };
        if signature.params != *params || signature.ret != *ret {
            self.fail(format!(
                "{at}: dynamic interface call signature disagrees with its table slot {slot}"
            ));
        }
        if param_modes.len() != params.len() {
            self.fail(format!(
                "{at}: dynamic interface call slot {slot} carries {} parameter modes for {} parameters",
                param_modes.len(),
                params.len()
            ));
        }
    }

    /// Verify `[MIR-3]` for the dynamic exclusivity brackets introduced by
    /// Phase 3 lowering.
    ///
    /// Access state is a multiset: an exact NLL region can end independently
    /// of a later-created region rooted at a different handle.  The runtime
    /// tracks that same per-object state, so treating MIR intervals as a strict
    /// lexical stack would reject valid aliasing code solely because its last
    /// uses are non-LIFO.  Repeated identical brackets are counted so nested
    /// reborrows still need a matching number of ends. The state is propagated
    /// through the reachable CFG and must close every access it opens: an end
    /// meets its access open, a return meets none open.
    ///
    /// An access begun on only some paths (a view made in one branch, kept
    /// past the join; D-455) sets its flag (a temporary named
    /// `ACCESS_FLAG_NAME`) as it begins and is ended behind a test of it. So a
    /// state also carries each such flag's value, a branch on a known one
    /// goes its one way, and a block keeps the states that reach it apart
    /// unless they are equal. A block reached by more than
    /// `MAX_ACCESS_STATES` different states fails as unequal states at a
    /// join did before.
    /// It intentionally does not decide whether an access *may* be elided:
    /// that requires `[EXC-3a]`'s safety side-table producer and reporting
    /// consumer, which are a separate implementation boundary.
    fn access_intervals(&mut self, body: &Body) {
        type Access = (Place, bool);
        /// `(ordinary, transferred)` per access: a transferred access is
        /// deliberately left open at this body boundary and will be closed by
        /// the caller through the returned payload.
        type Open = BTreeMap<Access, (usize, usize)>;
        #[derive(Clone, PartialEq, Eq)]
        struct State {
            open: Open,
            flags: BTreeMap<LocalId, bool>,
        }
        const MAX_ACCESS_STATES: usize = 256;
        let flag = |local: LocalId| {
            body.locals.get(local.0 as usize).is_some_and(|decl| {
                decl.kind == LocalKind::Temp && decl.name.as_deref() == Some(crate::ACCESS_FLAG_NAME)
            })
        };

        let mut seen: Vec<Vec<State>> = vec![Vec::new(); body.blocks.len()];
        let mut work = VecDeque::from([(BasicBlockId(0), State { open: Open::new(), flags: BTreeMap::new() })]);

        while let Some((block_id, mut state)) = work.pop_front() {
            let index = block_id.0 as usize;
            if index >= body.blocks.len() {
                // The structural pass above emits the precise bad-target
                // diagnostic. Do not duplicate it here.
                continue;
            }

            if seen[index].contains(&state) {
                continue;
            }
            if seen[index].len() == MAX_ACCESS_STATES {
                let previous = &seen[index][0].open;
                self.fail(format!(
                    "bb{index} receives incompatible dynamic-access states: previous {previous:?}, incoming {:?}",
                    state.open
                ));
                continue;
            }
            seen[index].push(state.clone());

            for stmt in &body.blocks[index].stmts {
                match &stmt.kind {
                    StmtKind::BeginAccess { place, mutable } => {
                        state.open.entry((place.clone(), *mutable)).or_default().0 += 1;
                    }
                    StmtKind::BeginAccessTransfer { place, mutable } => {
                        state.open.entry((place.clone(), *mutable)).or_default().1 += 1;
                    }
                    StmtKind::EndAccess { place, mutable } => {
                        let expected = (place.clone(), *mutable);
                        match state.open.get_mut(&expected) {
                            Some(counts) if counts.0 != 0 || counts.1 != 0 => {
                                if counts.0 != 0 {
                                    counts.0 -= 1;
                                } else {
                                    counts.1 -= 1;
                                }
                                if counts.0 == 0 && counts.1 == 0 {
                                    state.open.remove(&expected);
                                }
                            }
                            None | Some(_) => self.fail(format!(
                                "bb{index} ends dynamic access {expected:?}, but that access is not open"
                            )),
                        }
                    }
                    StmtKind::EndAccessTransfer { .. } => {}
                    StmtKind::Assign { place, rvalue } if flag(place.local) => {
                        state.flags.remove(&place.local);
                        if place.projection.is_empty()
                            && let Rvalue::Use(Operand::Const(Const::Bool(value))) = rvalue
                        {
                            state.flags.insert(place.local, *value);
                        }
                    }
                    _ => {}
                }
            }

            let mut enqueue = |target: BasicBlockId, state: &State| {
                if (target.0 as usize) < body.blocks.len() {
                    work.push_back((target, state.clone()));
                }
            };
            match &body.blocks[index].terminator {
                Terminator::Goto(target) => enqueue(*target, &state),
                Terminator::SwitchInt { discr, targets, otherwise } => {
                    // A branch on a flag this path set goes its one way.
                    let known = match discr {
                        Operand::Copy(place) | Operand::Move(place) if place.projection.is_empty() => {
                            state.flags.get(&place.local).map(|&value| i128::from(value))
                        }
                        _ => None,
                    };
                    match known {
                        Some(value) => {
                            let target = targets.iter().find(|(case, _)| *case == value).map_or(*otherwise, |(_, target)| *target);
                            enqueue(target, &state);
                        }
                        None => {
                            for (_, target) in targets {
                                enqueue(*target, &state);
                            }
                            enqueue(*otherwise, &state);
                        }
                    }
                }
                Terminator::Call { next, .. } | Terminator::Assert { next, .. } => enqueue(*next, &state),
                Terminator::Return => {
                    if state.open.values().any(|(ordinary, _)| *ordinary != 0) {
                        self.fail(format!(
                            "bb{index} terminates with dynamic accesses still open: {:?}",
                            state.open
                        ));
                    }
                }
                // `Unreachable` has no source-level exit or runtime path. It
                // commonly represents the impossible default edge of an
                // exhaustive match, so an access open only on that dead edge
                // cannot reach a caller and requires no runtime cleanup.
                Terminator::Unreachable => {}
            }
        }
    }

    /// `[EXC-11]` — hoisting metadata is not an advisory report. It names
    /// the exact compiler-internal bracket that code generation and safety
    /// inspection must agree on, so reject a stale or partial record before it
    /// can cross the backend boundary.
    fn hoisted_accesses(&mut self, body: &Body) {
        for record in &body.hoisted_accesses {
            let Some(preheader) = body.blocks.get(record.preheader.0 as usize) else {
                self.fail(format!(
                    "hoisted access preheader bb{} does not exist",
                    record.preheader.0
                ));
                continue;
            };
            if !preheader
                .stmts
                .get(record.preheader_statement)
                .is_some_and(|statement| {
                matches!(
                    &statement.kind,
                    StmtKind::BeginAccess { place, mutable }
                        if place == &record.place
                            && *mutable == record.mutable
                            && statement.span == record.span
                )
                })
            {
                self.fail(format!(
                    "hoisted access preheader bb{} statement {} lacks its matching begin",
                    record.preheader.0, record.preheader_statement
                ));
            }
            let Some(postheader) = body.blocks.get(record.postheader.0 as usize) else {
                self.fail(format!(
                    "hoisted access postheader bb{} does not exist",
                    record.postheader.0
                ));
                continue;
            };
            if !postheader.stmts.iter().any(|statement| {
                matches!(
                    &statement.kind,
                    StmtKind::EndAccess { place, mutable }
                        if place == &record.place
                            && *mutable == record.mutable
                            && statement.span == record.span
                )
            }) {
                self.fail(format!(
                    "hoisted access postheader bb{} lacks its matching end",
                    record.postheader.0
                ));
            }
        }
    }

    /// `[FN-5]` — a reservation forwarding record is trusted by borrow
    /// analysis, so its source must be one fresh mutable borrow and its
    /// binding must be one direct copy used by the named call. Endpoints are
    /// exclusive across records; this also rules out forwarding cycles.
    fn call_argument_bindings(&mut self, body: &Body) {
        let mut endpoints = BTreeSet::new();
        for record in &body.call_argument_bindings {
            let (Some(source), Some(binding), Some(block)) = (
                body.locals.get(record.source.0 as usize),
                body.locals.get(record.binding.0 as usize),
                body.blocks.get(record.call_block.0 as usize),
            ) else {
                self.fail(format!(
                    "call argument binding names invalid source _{}, binding _{}, or call bb{}",
                    record.source.0, record.binding.0, record.call_block.0
                ));
                continue;
            };
            if record.source == record.binding
                || !endpoints.insert(record.source)
                || !endpoints.insert(record.binding)
            {
                self.fail(format!(
                    "call argument binding from _{} to _{} repeats a local or forms a cycle",
                    record.source.0, record.binding.0
                ));
                continue;
            }
            if source.kind != LocalKind::Temp
                || binding.kind != LocalKind::User
                || source.ty != binding.ty
            {
                self.fail(format!(
                    "call argument binding from _{} to _{} must preserve the type of a fresh temporary",
                    record.source.0, record.binding.0
                ));
            }
            let mut source_writes = 0;
            let mut binding_writes = 0;
            let mut matching_source = 0;
            let mut matching_forward = 0;
            for block in &body.blocks {
                for stmt in &block.stmts {
                    if let StmtKind::Assign { place, rvalue } = &stmt.kind
                        && place.projection.is_empty()
                    {
                        if place.local == record.source {
                            source_writes += 1;
                            if matches!(rvalue, Rvalue::Ref { mutable: true, .. }) {
                                matching_source += 1;
                            }
                        }
                        if place.local == record.binding {
                            binding_writes += 1;
                            if matches!(rvalue, Rvalue::Use(Operand::Copy(place))
                                if place.projection.is_empty() && place.local == record.source)
                            {
                                matching_forward += 1;
                            }
                        }
                    }
                }
                if let Terminator::Call { dest, .. } = &block.terminator {
                    if dest.projection.is_empty() {
                        source_writes += usize::from(dest.local == record.source);
                        binding_writes += usize::from(dest.local == record.binding);
                    }
                }
            }
            if source_writes != 1 || matching_source != 1
                || binding_writes != 1 || matching_forward != 1
            {
                self.fail(format!(
                    "call argument binding from _{} to _{} lacks a unique mutable-reference producer and forwarding assignment",
                    record.source.0, record.binding.0
                ));
            }
            if !matches!(&block.terminator, Terminator::Call { args, .. }
                if args.iter().filter(|arg| matches!(arg, Operand::Copy(place)
                    if place.projection.is_empty() && place.local == record.binding)).count() == 1)
            {
                self.fail(format!(
                    "call argument binding _{} does not feed exactly one argument of bb{}",
                    record.binding.0, record.call_block.0
                ));
            }
        }
    }
}

/// Check one body. An empty result means it is well-formed.
pub fn verify(body: &Body) -> Vec<Violation> {
    let mut v = Verifier {
        symbol: &body.symbol,
        local_count: body.locals.len() as u32,
        block_count: body.blocks.len() as u32,
        violations: Vec::new(),
    };

    if body.locals.is_empty() {
        v.fail("a body must have at least the return local".to_string());
        return v.violations;
    }
    if body.arg_count + 1 > body.locals.len() {
        v.fail(format!(
            "arg_count is {} but the body has only {} locals",
            body.arg_count,
            body.locals.len()
        ));
    }
    if body.param_modes.len() != body.arg_count {
        v.fail(format!(
            "arg_count is {} but parameter-mode metadata has {} entries",
            body.arg_count,
            body.param_modes.len()
        ));
    }
    if body.blocks.is_empty() {
        v.fail("a body must have at least one basic block".to_string());
        return v.violations;
    }

    for (index, block) in body.blocks.iter().enumerate() {
        let at = format!("bb{index}");
        // `[CG-C-8]` — every statement carries the span of the source
        // construct that produced it, and the verifier checks it. Without a
        // span the backend emits no `#line`, so the debugger steps through
        // that statement as if it belonged to whatever came before, and
        // `tests/debug/`'s "one step advances one Ember statement" fails in a
        // way that points at the debugger rather than at the lowering.
        if block.terminator_span.is_dummy() {
            v.fail(format!("{at}: terminator has no source span"));
        }
        for stmt in &block.stmts {
            if stmt.span.is_dummy() && !stmt.kind.is_bookkeeping() {
                v.fail(format!("{at}: {} has no source span", stmt.kind.describe()));
            }
            match &stmt.kind {
                StmtKind::Assign { place, rvalue } => {
                    v.place(place, &at);
                    v.rvalue(rvalue, &at);
                }
                StmtKind::BeginAccess { place, .. }
                | StmtKind::BeginAccessTransfer { place, .. }
                | StmtKind::EndAccess { place, .. }
                | StmtKind::EndAccessTransfer { place, .. } => {
                    v.place(place, &at);
                }
                StmtKind::CheckedBinaryOp {
                    dest,
                    overflow,
                    lhs,
                    rhs,
                    ..
                } => {
                    v.place(dest, &at);
                    v.place(overflow, &at);
                    v.operand(lhs, &at);
                    v.operand(rhs, &at);
                }
                StmtKind::StorageLive(l) | StmtKind::StorageDead(l) => v.local(*l, &at),
                StmtKind::Drop { place, flag, .. } => {
                    v.place(place, &at);
                    if let Some(flag) = flag {
                        v.local(*flag, &at);
                    }
                }
                StmtKind::Nop => {}
            }
        }
        match &block.terminator {
            Terminator::Goto(bb) => v.target(*bb, &at),
            Terminator::SwitchInt {
                discr,
                targets,
                otherwise,
            } => {
                v.operand(discr, &at);
                for (_, bb) in targets {
                    v.target(*bb, &at);
                }
                v.target(*otherwise, &at);
            }
            Terminator::Call {
                func, args, dest, next,
            } => {
                for a in args {
                    v.operand(a, &at);
                }
                v.interface_call(func, &at);
                v.place(dest, &at);
                v.target(*next, &at);
            }
            Terminator::Assert {
                cond, msg, next, ..
            } => {
                v.operand(cond, &at);
                if let crate::AssertKind::Bounds { len, index } = msg {
                    v.operand(len, &at);
                    v.operand(index, &at);
                }
                if let crate::AssertKind::RefCellBorrow { file, line } = msg {
                    v.operand(file, &at);
                    v.operand(line, &at);
                }
                if let crate::AssertKind::Panic { message } = msg {
                    v.operand(message, &at);
                }
                v.target(*next, &at);
            }
            Terminator::Return | Terminator::Unreachable => {}
        }
    }

    // Run after structural place/target checks so the dataflow does not
    // produce duplicate diagnostics for malformed block references.
    v.access_intervals(body);
    v.hoisted_accesses(body);
    v.call_argument_bindings(body);

    v.violations
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        BasicBlock, CallArgumentBinding, HoistedAccess, HoistedAccessProof, LocalDecl, LocalKind, ParameterMode,
        Stmt,
    };
    use ember_span::Span;

    /// A body with one empty block and a real span on its terminator.
    fn body_with(stmts: Vec<Stmt>, terminator_span: Span) -> Body {
        Body {
            name: "t".to_string(),
            symbol: ember_branding::mangled("t"),
            is_unsafe: false,
            abi: None,
            overflow: ember_types::OverflowPolicy::Panic,
            fp: ember_types::FpMode::Strict,
            inline: ember_hir::InlineHint::default(),
            export_thread_policy: ember_hir::ExportThreadPolicy::Any,
            locals: vec![LocalDecl {
                ty: ember_types::TypeTable::new().1.void,
                name: None,
                kind: LocalKind::Return,
                span: Span::DUMMY,
            }],
            blocks: vec![BasicBlock {
                stmts,
                terminator: Terminator::Return,
                terminator_span,
            }],
            arg_count: 0,
            param_modes: Vec::new(),
            span: Span::DUMMY,
            borrows: None,
            sources: Vec::new(),
            is_lambda: false,
            emit_if_used: false,
            borrowed_params: Vec::new(),
            call_argument_bindings: Vec::new(),
            for_iterators: Vec::new(),
            callable_regions: None,
            closure_environment: None,
            closure_captures_by_move: false,
            class_owner: None,
            class_virtual_slot: None,
            is_abstract: false,
            is_extern_declaration: false,
            ffi_counted: None,
            mut_self: false,
            elided_accesses: Vec::new(),
            hoisted_accesses: Vec::new(),
            uncounted_handles: Vec::new(),
            removed_checks: Vec::new(),
            restrict_views: false,
            stack_buffers: Vec::new(),
        }
    }

    fn bound_call_body() -> Body {
        let span = Span::new(ember_span::FileId(0), 0, 1);
        let (mut types, common) = TypeTable::new();
        let mutable_ref = types.intern(TyKind::Ref { inner: common.i32, mutable: true });
        let mut body = body_with(Vec::new(), span);
        body.locals.extend([
            LocalDecl { ty: common.i32, kind: LocalKind::User, name: None, span },
            LocalDecl { ty: mutable_ref, kind: LocalKind::Temp, name: None, span },
            LocalDecl { ty: mutable_ref, kind: LocalKind::User, name: None, span },
        ]);
        body.blocks[0].stmts = vec![
            Stmt::new(StmtKind::Assign {
                place: Place::local(LocalId(2)),
                rvalue: Rvalue::Ref { place: Place::local(LocalId(1)), mutable: true },
            }, span),
            Stmt::new(StmtKind::Assign {
                place: Place::local(LocalId(3)),
                rvalue: Rvalue::Use(Operand::Copy(Place::local(LocalId(2)))),
            }, span),
        ];
        body.blocks[0].terminator = Terminator::Call {
            func: FuncRef::Direct { symbol: ember_branding::mangled("called"), latebound: false },
            args: vec![Operand::Copy(Place::local(LocalId(3)))],
            dest: Place::local(LocalId(0)),
            next: BasicBlockId(1),
        };
        body.blocks.push(BasicBlock {
            stmts: Vec::new(),
            terminator: Terminator::Return,
            terminator_span: span,
        });
        body.call_argument_bindings.push(CallArgumentBinding {
            source: LocalId(2), binding: LocalId(3), call_block: BasicBlockId(0),
        });
        body
    }

    #[test]
    fn call_argument_binding_requires_one_fresh_ref_and_exact_call() {
        let valid = bound_call_body();
        assert!(verify(&valid).is_empty(), "valid binding: {:?}", verify(&valid));

        let mut no_fresh_ref = bound_call_body();
        let StmtKind::Assign { rvalue, .. } = &mut no_fresh_ref.blocks[0].stmts[0].kind else { unreachable!() };
        *rvalue = Rvalue::Ref { place: Place::local(LocalId(1)), mutable: false };
        assert!(verify(&no_fresh_ref).iter().any(|v| v.message.contains("unique mutable-reference producer")));

        let mut wrong_call = bound_call_body();
        let Terminator::Call { args, .. } = &mut wrong_call.blocks[0].terminator else { unreachable!() };
        args[0] = Operand::Copy(Place::local(LocalId(2)));
        assert!(verify(&wrong_call).iter().any(|v| v.message.contains("does not feed exactly one argument")));

        let mut repeated = bound_call_body();
        repeated.call_argument_bindings.push(repeated.call_argument_bindings[0]);
        assert!(verify(&repeated).iter().any(|v| v.message.contains("repeats a local or forms a cycle")));
    }

    /// `[CG-C-8]` — the check must actually fire. A verifier check nobody can
    /// make fail is a comment.
    #[test]
    fn a_statement_without_a_span_is_a_violation() {
        let stmt = Stmt::new(
            StmtKind::Drop {
                place: Place::local(LocalId(0)),
                flag: None,
                scope_end: true,
            },
            Span::DUMMY,
        );
        let violations = verify(&body_with(
            vec![stmt],
            Span::new(ember_span::FileId(0), 0, 1),
        ));
        assert!(
            violations
                .iter()
                .any(|v| v.message.contains("no source span")),
            "expected a missing-span violation, got {violations:?}"
        );
    }

    #[test]
    fn a_terminator_without_a_span_is_a_violation() {
        let violations = verify(&body_with(Vec::new(), Span::DUMMY));
        assert!(
            violations
                .iter()
                .any(|v| v.message.contains("terminator has no source span")),
            "expected a missing-span violation, got {violations:?}"
        );
    }

    #[test]
    fn bookkeeping_statements_need_no_span() {
        let stmt = Stmt::new(StmtKind::StorageLive(LocalId(0)), Span::DUMMY);
        let violations = verify(&body_with(
            vec![stmt],
            Span::new(ember_span::FileId(0), 0, 1),
        ));
        assert!(violations.is_empty(), "got {violations:?}");
    }

    #[test]
    fn dynamic_access_must_close_before_return() {
        let span = Span::new(ember_span::FileId(0), 0, 1);
        let stmt = Stmt::new(
            StmtKind::BeginAccess {
                place: Place::local(LocalId(0)),
                mutable: true,
            },
            span,
        );
        let violations = verify(&body_with(vec![stmt], span));
        assert!(
            violations
                .iter()
                .any(|violation| violation.message.contains("still open")),
            "missing unclosed-access violation: {violations:?}"
        );
    }

    #[test]
    fn dynamic_access_may_remain_open_on_an_unreachable_path() {
        let span = Span::new(ember_span::FileId(0), 0, 1);
        let stmt = Stmt::new(
            StmtKind::BeginAccess {
                place: Place::local(LocalId(0)),
                mutable: true,
            },
            span,
        );
        let mut body = body_with(vec![stmt], span);
        body.blocks[0].terminator = Terminator::Unreachable;
        let violations = verify(&body);
        assert!(
            violations.is_empty(),
            "unreachable paths do not require cleanup: {violations:?}"
        );
    }

    #[test]
    fn corrupted_hoisted_access_metadata_is_a_violation() {
        let span = Span::new(ember_span::FileId(0), 0, 1);
        let place = Place::local(LocalId(0));
        let mut body = body_with(
            vec![
                Stmt::new(
                    StmtKind::BeginAccess {
                        place: place.clone(),
                        mutable: true,
                    },
                    span,
                ),
                Stmt::new(
                    StmtKind::EndAccess {
                        place: place.clone(),
                        mutable: true,
                    },
                    span,
                ),
            ],
            span,
        );
        body.hoisted_accesses.push(HoistedAccess {
            span,
            loop_span: span,
            preheader: BasicBlockId(1),
            preheader_statement: 0,
            postheader: BasicBlockId(0),
            place,
            mutable: true,
            proof: HoistedAccessProof::StableReceiverDirectCall,
        });

        let violations = verify(&body);
        assert!(
            violations
                .iter()
                .any(|violation| violation.message.contains("hoisted access preheader bb1")),
            "expected malformed hoist metadata to fail verification, got {violations:?}"
        );
    }

    #[test]
    fn transferred_dynamic_access_may_cross_a_return_boundary() {
        let span = Span::new(ember_span::FileId(0), 0, 1);
        let stmt = Stmt::new(
            StmtKind::BeginAccessTransfer {
                place: Place::local(LocalId(0)),
                mutable: true,
            },
            span,
        );
        let violations = verify(&body_with(vec![stmt], span));
        assert!(
            violations.is_empty(),
            "a caller must be able to close a returned access: {violations:?}"
        );
    }

    #[test]
    fn dynamic_accesses_may_end_in_exact_nll_order() {
        let span = Span::new(ember_span::FileId(0), 0, 1);
        let outer = Place::local(LocalId(0));
        let inner = outer.clone().field(0);
        let violations = verify(&body_with(
            vec![
                Stmt::new(
                    StmtKind::BeginAccess {
                        place: outer.clone(),
                        mutable: true,
                    },
                    span,
                ),
                Stmt::new(
                    StmtKind::BeginAccess {
                        place: inner.clone(),
                        mutable: true,
                    },
                    span,
                ),
                Stmt::new(
                    StmtKind::EndAccess {
                        place: outer,
                        mutable: true,
                    },
                    span,
                ),
                Stmt::new(
                    StmtKind::EndAccess {
                        place: inner,
                        mutable: true,
                    },
                    span,
                ),
            ],
            span,
        ));
        assert!(
            violations.is_empty(),
            "independent NLL ends must be valid: {violations:?}"
        );
    }

    /// An access begun on one branch and never ended reaches the return
    /// open on that path (states may differ at a join since D-455; what
    /// must hold is that every path closes what it opened).
    #[test]
    fn an_access_open_on_one_path_must_still_close() {
        let span = Span::new(ember_span::FileId(0), 0, 1);
        let (_, common) = TypeTable::new();
        let mut body = body_with(Vec::new(), span);
        body.arg_count = 1;
        body.param_modes = vec![ParameterMode::Borrow];
        body.locals.push(LocalDecl {
            ty: common.bool_,
            name: Some("choose".to_string()),
            kind: LocalKind::Arg,
            span,
        });
        body.blocks = vec![
            BasicBlock {
                stmts: Vec::new(),
                terminator: Terminator::SwitchInt {
                    discr: Operand::Copy(Place::local(LocalId(1))),
                    targets: vec![(1, BasicBlockId(1))],
                    otherwise: BasicBlockId(2),
                },
                terminator_span: span,
            },
            BasicBlock {
                stmts: vec![Stmt::new(
                    StmtKind::BeginAccess {
                        place: Place::local(LocalId(0)),
                        mutable: true,
                    },
                    span,
                )],
                terminator: Terminator::Goto(BasicBlockId(3)),
                terminator_span: span,
            },
            BasicBlock {
                stmts: Vec::new(),
                terminator: Terminator::Goto(BasicBlockId(3)),
                terminator_span: span,
            },
            BasicBlock {
                stmts: Vec::new(),
                terminator: Terminator::Return,
                terminator_span: span,
            },
        ];
        let violations = verify(&body);
        assert!(
            violations.iter().any(|violation| violation.message.contains("still open")),
            "missing open-access violation: {violations:?}"
        );
    }

    /// D-455 — an access begun on one branch sets its flag; past the join,
    /// its end runs behind a test of the flag. Every path closes what it
    /// opened, so the body verifies; without the test (an end on every path)
    /// the path that never began it ends it, which is reported.
    #[test]
    fn a_flagged_access_ends_only_where_it_began() {
        let span = Span::new(ember_span::FileId(0), 0, 1);
        let (_, common) = TypeTable::new();
        let build = |tested: bool| {
            let mut body = body_with(Vec::new(), span);
            body.arg_count = 1;
            body.param_modes = vec![ParameterMode::Borrow];
            body.locals.push(LocalDecl { ty: common.bool_, name: Some("choose".to_string()), kind: LocalKind::Arg, span });
            body.locals.push(LocalDecl {
                ty: common.bool_,
                name: Some(crate::ACCESS_FLAG_NAME.to_string()),
                kind: LocalKind::Temp,
                span,
            });
            let flag = |value: bool| {
                Stmt::new(
                    StmtKind::Assign {
                        place: Place::local(LocalId(2)),
                        rvalue: Rvalue::Use(Operand::Const(Const::Bool(value))),
                    },
                    span,
                )
            };
            let access = Place::local(LocalId(0));
            let end = Stmt::new(StmtKind::EndAccess { place: access.clone(), mutable: true }, span);
            body.blocks = vec![
                BasicBlock {
                    stmts: vec![flag(false)],
                    terminator: Terminator::SwitchInt {
                        discr: Operand::Copy(Place::local(LocalId(1))),
                        targets: vec![(1, BasicBlockId(1))],
                        otherwise: BasicBlockId(2),
                    },
                    terminator_span: span,
                },
                BasicBlock {
                    stmts: vec![Stmt::new(StmtKind::BeginAccess { place: access.clone(), mutable: true }, span), flag(true)],
                    terminator: Terminator::Goto(BasicBlockId(3)),
                    terminator_span: span,
                },
                BasicBlock { stmts: Vec::new(), terminator: Terminator::Goto(BasicBlockId(3)), terminator_span: span },
                BasicBlock {
                    stmts: if tested { Vec::new() } else { vec![end.clone()] },
                    terminator: if tested {
                        Terminator::SwitchInt {
                            discr: Operand::Copy(Place::local(LocalId(2))),
                            targets: vec![(0, BasicBlockId(5))],
                            otherwise: BasicBlockId(4),
                        }
                    } else {
                        Terminator::Goto(BasicBlockId(5))
                    },
                    terminator_span: span,
                },
                BasicBlock { stmts: vec![end, flag(false)], terminator: Terminator::Goto(BasicBlockId(5)), terminator_span: span },
                BasicBlock { stmts: Vec::new(), terminator: Terminator::Return, terminator_span: span },
            ];
            body
        };
        let violations = verify(&build(true));
        assert!(violations.is_empty(), "a flagged access must verify: {violations:?}");
        let violations = verify(&build(false));
        assert!(
            violations.iter().any(|violation| violation.message.contains("is not open")),
            "an end on a path that never began the access must be reported: {violations:?}"
        );
    }

    #[test]
    fn parameter_mode_metadata_is_a_verified_callable_fact() {
        let span = Span::new(ember_span::FileId(0), 0, 1);
        let mut body = body_with(Vec::new(), span);
        body.arg_count = 1;
        body.locals.push(LocalDecl {
            ty: ember_types::TypeTable::new().1.i32,
            name: Some("value".to_string()),
            kind: LocalKind::Arg,
            span,
        });
        let violations = verify(&body);
        assert!(
            violations
                .iter()
                .any(|violation| violation.message.contains("parameter-mode metadata")),
            "missing parameter-mode metadata crossed verification: {violations:?}"
        );
    }

    #[test]
    fn dynamic_interface_calls_must_target_their_declared_table_slot() {
        let span = Span::new(ember_span::FileId(0), 0, 1);
        let (_, common) = TypeTable::new();
        let mut body = body_with(Vec::new(), span);
        body.blocks[0].terminator = Terminator::Call {
            func: FuncRef::Interface {
                interfaces: vec![ember_span::Symbol::intern("Drawable")],
                interface: ember_span::Symbol::intern("Drawable"),
                slot: 1,
                params: Vec::new(),
                param_modes: Vec::new(),
                ret: common.i32,
                layout: vec![Some(ember_hir::InterfaceSlot {
                    params: Vec::new(),
                    ret: common.i32,
                })],
                class_handle: false,
            },
            args: Vec::new(),
            dest: Place::local(LocalId(0)),
            next: BasicBlockId(1),
        };
        body.blocks.push(BasicBlock {
            stmts: Vec::new(),
            terminator: Terminator::Return,
            terminator_span: span,
        });
        let violations = verify(&body);
        assert!(
            violations.iter().any(|violation| violation
                .message
                .contains("targets non-callable slot 1")),
            "missing dynamic-table violation: {violations:?}"
        );
    }

    #[test]
    fn the_codegen_boundary_rejects_structurally_unverified_mir() {
        let bodies = vec![body_with(Vec::new(), Span::DUMMY)];
        let (types, _) = TypeTable::new();
        let Err(violations) = for_codegen(&bodies, &types) else {
            panic!("structurally invalid MIR crossed the codegen boundary");
        };
        assert!(
            violations
                .iter()
                .any(|v| v.message.contains("terminator has no source span"))
        );
    }

    #[test]
    fn the_codegen_boundary_requires_callable_region_metadata() {
        let span = Span::new(ember_span::FileId(0), 0, 1);
        let bodies = vec![body_with(Vec::new(), span)];
        let (types, _) = TypeTable::new();
        let Err(violations) = for_codegen(&bodies, &types) else {
            panic!("MIR without callable-region metadata crossed the codegen boundary");
        };
        assert!(
            violations
                .iter()
                .any(|v| v.message.contains("metadata is missing"))
        );
    }

    #[test]
    fn the_codegen_boundary_rejects_a_corrupt_callable_region_fingerprint() {
        let span = Span::new(ember_span::FileId(0), 0, 1);
        let mut body = body_with(Vec::new(), span);
        let mut metadata = crate::CallableRegionMetadata::new(
            crate::CallableAccessSummary::Fields(Vec::new()),
            None,
        );
        metadata.corrupt_fingerprint_for_test();
        body.callable_regions = Some(metadata);
        let bodies = vec![body];
        let (types, _) = TypeTable::new();
        let Err(violations) = for_codegen(&bodies, &types) else {
            panic!("corrupt callable-region metadata crossed the codegen boundary");
        };
        assert!(
            violations
                .iter()
                .any(|v| v.message.contains("stale or corrupt"))
        );
    }

    #[test]
    fn callable_region_fingerprint_changes_with_the_contract() {
        let empty = crate::CallableRegionMetadata::new(
            crate::CallableAccessSummary::Fields(Vec::new()),
            None,
        );
        let reads_first = crate::CallableRegionMetadata::new(
            crate::CallableAccessSummary::Fields(vec![crate::ParameterFieldAccess {
                argument: 0,
                projection: vec![Projection::Field(0)],
                operations: vec![crate::RegionAccessKind::Read],
            }]),
            None,
        );
        let reads_second = crate::CallableRegionMetadata::new(
            crate::CallableAccessSummary::Fields(vec![crate::ParameterFieldAccess {
                argument: 0,
                projection: vec![Projection::Field(1)],
                operations: vec![crate::RegionAccessKind::Read],
            }]),
            None,
        );
        let returns_first = crate::CallableRegionMetadata::new(
            crate::CallableAccessSummary::Fields(Vec::new()),
            Some(crate::ResultProvenanceSummary {
                fields: vec![crate::ResultFieldProvenance {
                    result_projection: vec![Projection::Field(0)],
                    sources: vec![crate::ResultRegionSource::View {
                        argument: 0,
                        projection: vec![Projection::Field(0)],
                    }],
                }],
            }),
        );
        let returns_second = crate::CallableRegionMetadata::new(
            crate::CallableAccessSummary::Fields(Vec::new()),
            Some(crate::ResultProvenanceSummary {
                fields: vec![crate::ResultFieldProvenance {
                    result_projection: vec![Projection::Field(0)],
                    sources: vec![crate::ResultRegionSource::View {
                        argument: 1,
                        projection: vec![Projection::Field(0)],
                    }],
                }],
            }),
        );
        let reordered = crate::CallableRegionMetadata::new(
            crate::CallableAccessSummary::Fields(vec![
                crate::ParameterFieldAccess {
                    argument: 1,
                    projection: vec![Projection::Field(1)],
                    operations: vec![
                        crate::RegionAccessKind::Write,
                        crate::RegionAccessKind::Read,
                    ],
                },
                crate::ParameterFieldAccess {
                    argument: 0,
                    projection: vec![Projection::Field(0)],
                    operations: vec![crate::RegionAccessKind::Read],
                },
            ]),
            None,
        );
        let canonical = crate::CallableRegionMetadata::new(
            crate::CallableAccessSummary::Fields(vec![
                crate::ParameterFieldAccess {
                    argument: 0,
                    projection: vec![Projection::Field(0)],
                    operations: vec![crate::RegionAccessKind::Read],
                },
                crate::ParameterFieldAccess {
                    argument: 1,
                    projection: vec![Projection::Field(1)],
                    operations: vec![
                        crate::RegionAccessKind::Read,
                        crate::RegionAccessKind::Write,
                    ],
                },
            ]),
            None,
        );
        assert_ne!(empty.fingerprint(), reads_first.fingerprint());
        assert_ne!(reads_first.fingerprint(), reads_second.fingerprint());
        assert_ne!(returns_first.fingerprint(), returns_second.fingerprint());
        assert_eq!(reordered, canonical);
        assert_eq!(
            reads_first.fingerprint(),
            crate::CallableRegionMetadata::new(
                crate::CallableAccessSummary::Fields(vec![crate::ParameterFieldAccess {
                    argument: 0,
                    projection: vec![Projection::Field(0)],
                    operations: vec![crate::RegionAccessKind::Read],
                }]),
                None,
            )
            .fingerprint()
        );

        // `[MIR-REG-1]` — the interface artifact receives canonical bytes,
        // not a Rust-memory snapshot. A damaged trailing identity must be
        // refused before a caller treats the decoded record as a contract.
        let encoded = returns_second.to_interface_bytes().unwrap();
        assert_eq!(
            crate::CallableRegionMetadata::from_interface_bytes(&encoded).unwrap(),
            returns_second
        );
        let mut corrupt = encoded;
        let last = corrupt.len() - 1;
        corrupt[last] ^= 1;
        assert!(matches!(
            crate::CallableRegionMetadata::from_interface_bytes(&corrupt),
            Err(crate::CallableRegionMetadataCodecError::StaleFingerprint)
        ));
    }
}

/// Verify every body, panicking on the first violation. Called from the driver
/// in debug builds of the compiler.
pub fn verify_all(bodies: &[Body]) {
    let violations: Vec<Violation> = bodies.iter().flat_map(verify).collect();
    assert!(
        violations.is_empty(),
        "MIR verification failed:\n{}",
        violations
            .iter()
            .map(|v| format!("  {}: {}", v.body, v.message))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

// ---------------------------------------------------------------------------
// The view invariant
// ---------------------------------------------------------------------------

/// **Every view in the MIR must have arrived from a borrow.**
///
/// This is the structural backstop for D-022. `[SPN-1]` reads like a coercion
/// — "`Array[T]` coerces to `Span[T]` at borrow sites" — and it was
/// implemented like one, so `v: Span[i32] = a` produced a view with no
/// `Rvalue::Ref` anywhere near it. `collect_loans` looks for `Rvalue::Ref` and
/// found none, so the borrow checker saw no borrow, `a.push(…)` was permitted,
/// the push reallocated, and `v` pointed into freed memory. A use-after-free
/// with no `unsafe` in the program, which is precisely what `[PHIL-10]` says
/// cannot happen.
///
/// The lesson generalises past spans: **a construct that changes lifetime or
/// aliasing must not be represented as a conversion.** So rather than trusting
/// that every future view-producing path remembers to take a borrow, the
/// finished MIR is checked for the shape a forgotten one leaves behind.
///
/// A view-typed place may be written only by a *provenance-carrying* rvalue:
///
/// * `Ref` — the borrow itself, which is where provenance begins;
/// * `Use` of a place — a copy or move of a view that already has provenance
///   (`[SPN-3]` makes `Span[T]` `Copy`);
/// * `Use` of a string constant — `[LEX-20]` gives literals static region;
/// * `Aggregate`/`Repeat` — a view struct or tuple, whose provenance is its
///   operands' (`[LT-2]`).
///
/// and never by one that manufactures a value out of representation: `Cast`,
/// `BinaryOp`, `UnaryOp` or `Discriminant`. None of those has anything to
/// point into, so a view coming out of one is a view with no loan behind it.
///
/// Separately, `Builtin::SpanFrom` is the one producer, and its argument must
/// be a reference: the borrow has to be *written down* in the IR, because an
/// implied one is exactly what the analyses cannot see.
///
/// This runs in debug builds of the compiler beside [`verify_all`]. It is a
/// compiler-internal invariant, not a diagnostic — a violation is a bug in a
/// lowering, and the program that provoked it may be perfectly correct Ember.
pub fn verify_views(body: &Body, types: &TypeTable) -> Vec<Violation> {
    let mut violations = Vec::new();
    let mut fail = |message: String| {
        violations.push(Violation {
            body: body.symbol.to_string(),
            message,
        });
    };

    for (index, block) in body.blocks.iter().enumerate() {
        for stmt in &block.stmts {
            let StmtKind::Assign { place, rvalue } = &stmt.kind else {
                continue;
            };
            if !types.is_view(place_ty(body, types, place)) {
                continue;
            }
            let manufactured = match rvalue {
                // `[CLS-4]` — this is a borrow-preserving pointer adjustment
                // from a derived class receiver to the base receiver of an
                // inherited `mut self` method. Unlike numeric and owning
                // class casts, it does not manufacture a view without a
                // source loan.
                Rvalue::Cast {
                    kind:
                        crate::CastKind::ClassUpcastBorrowed
                        | crate::CastKind::InterfaceUpcast { .. }
                        | crate::CastKind::ClassInterfaceUpcast { .. },
                    ..
                } => None,
                Rvalue::Cast { .. } => Some("a cast"),
                Rvalue::BinaryOp { .. } => Some("an arithmetic operation"),
                Rvalue::UnaryOp { .. } => Some("a unary operation"),
                Rvalue::Discriminant(_) => Some("an enum discriminant"),
                Rvalue::Use(Operand::Const(c)) if !matches!(c, Const::Str(_) | Const::CStrLiteral(_)) => {
                    Some("a non-string constant")
                }
                _ => None,
            };
            if let Some(what) = manufactured {
                fail(format!(
                    "bb{index}: a view is assigned from {what}, which carries no borrow \
                     — a view-producing path must take one (see D-022)"
                ));
            }
        }

        if let Terminator::Call { func, args, .. } = &block.terminator {
            // `StringAsStr` and `ArenaAlloc` ride the same shape: the returned
            // view points into storage owned by the first argument, so that
            // argument must be an explicit borrow too (D-037, `[LT-4]`).
            let FuncRef::Builtin { which, .. } = func else {
                continue;
            };
            if !matches!(
                which,
                Builtin::SpanFrom { .. }
                    | Builtin::StringAsStr
                    | Builtin::CStringAsCStr
                    | Builtin::ArenaArrayWithCapacity { .. }
                    | Builtin::ArenaMapWithCapacity { .. }
                    | Builtin::ArenaAlloc { .. }
                    | Builtin::ArenaAllocUninit { .. }
                    | Builtin::ArenaAllocArrayZeroed { .. }
                    | Builtin::ArenaAllocArrayDefault { .. }
                    | Builtin::FixedArenaAlloc { .. }
                    | Builtin::ScopedArenaAlloc { .. }
                    | Builtin::ArenaScope { .. }
            ) {
                continue;
            };
            let borrowed = match args.first() {
                Some(Operand::Copy(p) | Operand::Move(p)) => {
                    matches!(types.kind(place_ty(body, types, p)), TyKind::Ref { .. })
                }
                _ => false,
            };
            if !borrowed {
                fail(format!(
                    "bb{index}: a view-producing builtin is applied to something that is not a \
                     reference — the borrow the view is built from must be explicit \
                     in the IR, or `collect_loans` cannot see it (see D-022)"
                ));
            }
        }
    }
    violations
}

/// `[TYP-22]` — an interface upcast is the one cast that transports an
/// existing borrow into the erased `{data*, vtable*}` carrier.  It must remain
/// structurally narrow: accepting an arbitrary cast here would recreate
/// D-022's failure mode, where a lifetime-carrying value reached later phases
/// without a source loan the analyses could see.
///
/// The type checker is authoritative for conformance and the declaration
/// layout. This verifier protects the HIR/MIR/codegen boundary against a
/// malformed lowering by checking the facts that must be visible in MIR:
/// supported concrete source, matching borrow mutability, one matching target
/// interface, matched adapter/layout slots, and no move-only receiver.
pub fn verify_interface_upcasts(body: &Body, types: &TypeTable) -> Vec<Violation> {
    let mut violations = Vec::new();
    let mut fail = |message: String| {
        violations.push(Violation {
            body: body.symbol.to_string(),
            message,
        });
    };

    for (block_index, block) in body.blocks.iter().enumerate() {
        for statement in &block.stmts {
            if let StmtKind::Assign {
                place,
                rvalue:
                    Rvalue::Cast {
                        kind:
                            crate::CastKind::ClassInterfaceUpcast {
                                concrete,
                                interface,
                                layout,
                                implementations,
                            },
                        operand,
                        to,
                    },
            } = &statement.kind
            {
                let at = format!("bb{block_index}: class interface upcast");
                let destination = place_ty(body, types, place);
                if *to != destination {
                    fail(format!("{at} cast type disagrees with its destination place"));
                    continue;
                }
                let source = match operand {
                    Operand::Copy(source) | Operand::Move(source) => place_ty(body, types, source),
                    Operand::Const(_) => {
                        fail(format!("{at} must preserve a class handle, not a constant"));
                        continue;
                    }
                };
                let source_concrete = match types.kind(source) {
                    TyKind::Class(_) => Some(source),
                    TyKind::Ref { mutable: true, inner }
                        if matches!(types.kind(*inner), TyKind::Class(_)) =>
                    {
                        Some(*inner)
                    }
                    _ => None,
                };
                if source_concrete != Some(*concrete) {
                    fail(format!("{at} concrete metadata disagrees with its class source"));
                    continue;
                }
                let target_interface = match types.kind(destination) {
                    TyKind::ClassInterface(target) => Some(*target),
                    TyKind::Ref { mutable: true, inner }
                        if matches!(types.kind(*inner), TyKind::ClassInterface(_)) =>
                    {
                        let TyKind::ClassInterface(target) = types.kind(*inner) else {
                            unreachable!("class-interface target was checked above");
                        };
                        Some(*target)
                    }
                    _ => None,
                };
                if target_interface != Some(*interface) {
                    fail(format!("{at} interface metadata disagrees with its target type"));
                    continue;
                }
                if layout.len() != implementations.len() {
                    fail(format!("{at} has mismatched table-layout and adapter lengths"));
                    continue;
                }
                for (slot, (layout, implementation)) in layout.iter().zip(implementations).enumerate() {
                    match (layout, implementation) {
                        (None, None) => {}
                        (Some(_), Some(implementation))
                            if implementation.receiver != ember_hir::Mode::Owned => {}
                        (None, Some(_)) => {
                            fail(format!("{at} supplies an adapter for non-callable slot {slot}"));
                        }
                        (Some(_), None) => {
                            fail(format!("{at} omits an adapter for callable slot {slot}"));
                        }
                        (Some(_), Some(_)) => {
                            fail(format!("{at} supplies an owned receiver for slot {slot}"));
                        }
                    }
                }
                continue;
            }
            let StmtKind::Assign {
                place,
                rvalue:
                    Rvalue::Cast {
                        kind:
                            crate::CastKind::InterfaceUpcast {
                                concrete,
                                interfaces,
                                layout,
                                implementations,
                            },
                        operand,
                        to,
                    },
            } = &statement.kind
            else {
                continue;
            };

            let at = format!("bb{block_index}: dynamic interface upcast");
            let destination = place_ty(body, types, place);
            if *to != destination {
                fail(format!("{at} cast type disagrees with its destination place"));
                continue;
            }
            let source = match operand {
                Operand::Copy(source) | Operand::Move(source) => place_ty(body, types, source),
                Operand::Const(_) => {
                    fail(format!("{at} must preserve a borrowed class place, not a constant"));
                    continue;
                }
            };
            let (TyKind::Ref { mutable: source_mutable, inner: source_inner },
                TyKind::Ref { mutable: target_mutable, inner: target_inner }) =
                (types.kind(source), types.kind(destination))
            else {
                fail(format!("{at} must convert a concrete borrow to `ref dyn Interface`"));
                continue;
            };
            if source_mutable != target_mutable {
                fail(format!("{at} changes borrow mutability"));
                continue;
            }
            let supported_concrete = match types.kind(*concrete) {
                TyKind::Class(_) => true,
                _ => is_source_value_payload(types, *concrete),
            };
            if source_inner != concrete || !supported_concrete {
                fail(format!("{at} concrete metadata disagrees with its source type"));
                continue;
            }
            if !matches!(types.kind(*target_inner), TyKind::Dyn { interfaces: target_interfaces }
                if target_interfaces == interfaces)
            {
                fail(format!("{at} interface metadata disagrees with its target type"));
                continue;
            }
            if layout.len() != implementations.len() {
                fail(format!("{at} has mismatched table-layout and adapter lengths"));
                continue;
            }
            for (slot, (layout, implementation)) in layout.iter().zip(implementations).enumerate() {
                match (layout, implementation) {
                    (None, None) => {}
                    (Some(_), Some(implementation))
                        if implementation.receiver != ember_hir::Mode::Owned => {}
                    (None, Some(_)) => {
                        fail(format!("{at} supplies an adapter for non-callable slot {slot}"));
                    }
                    (Some(_), None) => {
                        fail(format!("{at} omits an adapter for callable slot {slot}"));
                    }
                    (Some(_), Some(_)) => {
                        fail(format!("{at} supplies an owned receiver for slot {slot}"));
                    }
                }
            }
        }

        let Terminator::Call {
            func:
                FuncRef::DynBoxNew {
                    concrete,
                    boxed,
                    interfaces,
                    layout,
                    implementations,
                },
            args,
            dest,
            ..
        } = &block.terminator
        else {
            continue;
        };
        let at = format!("bb{block_index}: dynamic interface box");
        let valid_box = match types.kind(*boxed) {
            TyKind::Struct(id) => match types.compiler_box_inner(*id) {
                Some(inner) => {
                    matches!(types.kind(inner), TyKind::Dyn { interfaces: target_interfaces }
                        if target_interfaces == interfaces)
                }
                None => false,
            },
            _ => false,
        };
        if place_ty(body, types, dest) != *boxed || !valid_box {
            fail(format!("{at} metadata disagrees with its destination type"));
            continue;
        }
        let supported_concrete = matches!(types.kind(*concrete), TyKind::Class(_))
            || is_source_value_payload(types, *concrete);
        let valid_payload = matches!(args.as_slice(), [Operand::Move(source)]
            if place_ty(body, types, source) == *concrete)
            || matches!(args.as_slice(), [Operand::Copy(source)]
                if types.is_copy(*concrete) && place_ty(body, types, source) == *concrete)
            || matches!(args.as_slice(), [Operand::Const(constant)]
                if is_copy_scalar_constant(types, constant, *concrete));
        if !supported_concrete || !valid_payload {
            fail(format!(
                "{at} must move one concrete struct, enum, scalar, or class payload"
            ));
            continue;
        }
        if layout.len() != implementations.len() {
            fail(format!("{at} has mismatched table-layout and adapter lengths"));
            continue;
        }
        for (slot, (layout, implementation)) in layout.iter().zip(implementations).enumerate() {
            match (layout, implementation) {
                (None, None) => {}
                (Some(_), Some(implementation))
                    if implementation.receiver != ember_hir::Mode::Owned => {}
                (None, Some(_)) => {
                    fail(format!("{at} supplies an adapter for non-callable slot {slot}"));
                }
                (Some(_), None) => {
                    fail(format!("{at} omits an adapter for callable slot {slot}"));
                }
                (Some(_), Some(_)) => {
                    fail(format!("{at} supplies an owned receiver for slot {slot}"));
                }
            }
        }
    }
    violations
}

fn is_source_value_payload(types: &TypeTable, ty: Ty) -> bool {
    if types.is_primitive_scalar(ty) {
        return true;
    }
    match types.kind(ty) {
        TyKind::Struct(id) => {
            types.struct_def(*id).origin.is_none()
                || types.struct_def(*id).declaring_module != usize::MAX
        }
        TyKind::Enum(_) => true,
        _ => false,
    }
}

fn is_copy_scalar_constant(types: &TypeTable, constant: &Const, concrete: Ty) -> bool {
    if !types.is_primitive_scalar(concrete) || !types.is_copy(concrete) {
        return false;
    }
    match constant {
        Const::Int { ty, .. } | Const::Float { ty, .. } => *ty == concrete,
        Const::Bool(_) => matches!(types.kind(concrete), TyKind::Bool),
        _ => false,
    }
}

/// Where a place lands, following its projections. A projection that does not
/// apply leaves the type alone: the type checker has already rejected such a
/// program, and the verifier only has to stay on its feet.
fn place_ty(body: &Body, types: &TypeTable, place: &Place) -> Ty {
    let mut ty = body.local(place.local).ty;
    let mut variant = None;
    for projection in &place.projection {
        match (projection, types.kind(ty)) {
            (Projection::Downcast(v), TyKind::Enum(_)) => variant = Some(*v),
            (Projection::Field(i), TyKind::Enum(id)) => {
                let Some(v) = variant else { continue };
                let fields = &types.enum_def(*id).variants[v].fields;
                ty = fields.get(*i).map(|f| f.ty).unwrap_or(ty);
                variant = None;
            }
            (Projection::Field(i), TyKind::Struct(id)) => {
                let fields = &types.struct_def(*id).fields;
                ty = fields.get(*i).map(|f| f.ty).unwrap_or(ty);
            }
            (Projection::Field(i), TyKind::Class(id)) => {
                ty = types.class_field_at(*id, *i).map(|f| f.ty).unwrap_or(ty);
            }
            (Projection::Field(i), TyKind::Tuple(items)) => {
                ty = items.get(*i).copied().unwrap_or(ty);
            }
            (
                Projection::Index(_) | Projection::ConstIndex(_) | Projection::Column(_),
                TyKind::Array { elem, .. } | TyKind::Vec { elem, .. } | TyKind::Span { elem, .. },
            ) => ty = *elem,
            (
                Projection::Index(_) | Projection::ConstIndex(_) | Projection::Column(_),
                TyKind::Ptr { inner, .. },
            ) => ty = *inner,
            (Projection::Deref, TyKind::Ref { inner, .. } | TyKind::Ptr { inner, .. }) => {
                ty = *inner
            }
            _ => {}
        }
    }
    ty
}

/// Verify the view invariant across every body, panicking on the first
/// violation. Called from the driver in debug builds, after the borrow checker
/// has run on the same MIR.
pub fn verify_views_all(bodies: &[Body], types: &TypeTable) {
    let violations: Vec<Violation> = bodies.iter().flat_map(|b| verify_views(b, types)).collect();
    assert!(
        violations.is_empty(),
        "MIR view verification failed:\n{}",
        violations
            .iter()
            .map(|v| format!("  {}: {}", v.body, v.message))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// Run the dynamic-interface-upcast boundary verifier over every lowered
/// body. Like [`verify_views_all`], violations are compiler bugs, not source
/// diagnostics: safe source should never be blamed for malformed MIR.
pub fn verify_interface_upcasts_all(bodies: &[Body], types: &TypeTable) {
    let violations: Vec<Violation> = bodies
        .iter()
        .flat_map(|body| verify_interface_upcasts(body, types))
        .collect();
    assert!(
        violations.is_empty(),
        "MIR dynamic-interface-upcast verification failed:\n{}",
        violations
            .iter()
            .map(|violation| format!("  {}: {}", violation.body, violation.message))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// `[MIR-REG-1]` / `[VERIFY-3]` — the artifact crossing the final MIR
/// boundary must carry callable-region metadata whose deterministic identity
/// still matches its contents. Semantic agreement with field accesses is
/// rederived by `ember_analysis`; this lower-level check makes omission or
/// byte-level staleness unconditionally fatal before code generation.
pub fn verify_callable_region_metadata(body: &Body) -> Vec<Violation> {
    match &body.callable_regions {
        None => vec![Violation {
            body: body.symbol.clone(),
            message: "callable-region metadata is missing".to_string(),
        }],
        Some(metadata) if !metadata.fingerprint_is_valid() => vec![Violation {
            body: body.symbol.clone(),
            message: "callable-region metadata fingerprint is stale or corrupt".to_string(),
        }],
        Some(_) => Vec::new(),
    }
}

/// `[IMP-7]` / `[VERIFY-3]` — establish the code-generation boundary.
///
/// This runs in every compiler profile, after the final MIR body-selection
/// transformation. A verifier failure is a compiler defect, so the caller
/// reports the collected internal violations rather than translating
/// untrusted MIR.
pub fn for_codegen<'a>(
    bodies: &'a [Body],
    types: &'a TypeTable,
) -> Result<VerifiedMir<'a>, Vec<Violation>> {
    let mut violations: Vec<Violation> = bodies.iter().flat_map(verify).collect();
    violations.extend(bodies.iter().flat_map(|body| verify_views(body, types)));
    violations.extend(
        bodies
            .iter()
            .flat_map(|body| verify_interface_upcasts(body, types)),
    );
    violations.extend(bodies.iter().flat_map(verify_callable_region_metadata));
    if violations.is_empty() {
        Ok(VerifiedMir { bodies, types })
    } else {
        Err(violations)
    }
}

#[cfg(test)]
mod view_invariant_tests {
    use super::*;
    use crate::{BasicBlock, LocalDecl, LocalKind, Stmt};
    use ember_span::{Span, Symbol};
    use ember_types::{FieldDef, FieldVis, StructDef, TypeTable};

    /// A closure environment is a struct of `[CLO-2]` capture borrows, so
    /// `is_view` holds of it and `verify_views` governs how it may be built.
    /// That is the whole of item 30's closure invariant: it needed no new
    /// machinery, because "an environment is a value carrying borrows" is the
    /// same statement the view invariant already makes.
    ///
    /// Asserted rather than assumed. The first version of this reasoning was
    /// right and unproven, which is the state D-022 was in.
    fn env_body(rvalue: Rvalue) -> (Body, TypeTable) {
        let (mut types, common) = TypeTable::new();
        let i32_ty = common.i32;
        let borrow = types.intern(TyKind::Ref {
            mutable: false,
            inner: i32_ty,
        });
        let env = types.add_struct(StructDef {
            name: Symbol::intern("closure0_env"),
            fields: vec![FieldDef {
                name: Symbol::intern("n"),
                ty: borrow,
                span: Span::DUMMY,
                ty_span: Span::DUMMY,
                has_default: false,
                is_let: false,
                read_only_outside: false,
                vis: FieldVis::Private,
            }],
            span: Span::DUMMY,
            derives_copy: true,
            has_drop: false,
            drops_fields: true,
            origin: None,
            declaring_module: 0,
        });
        let env_ty = types.intern(TyKind::Struct(env));
        let span = Span::new(ember_span::FileId(0), 0, 1);
        let body = Body {
            name: "t".to_string(),
            symbol: ember_branding::mangled("t"),
            is_unsafe: false,
            abi: None,
            overflow: ember_types::OverflowPolicy::Panic,
            fp: ember_types::FpMode::Strict,
            inline: ember_hir::InlineHint::default(),
            export_thread_policy: ember_hir::ExportThreadPolicy::Any,
            locals: vec![
                LocalDecl {
                    ty: env_ty,
                    name: None,
                    kind: LocalKind::Return,
                    span: Span::DUMMY,
                },
                LocalDecl {
                    ty: common.i32,
                    name: None,
                    kind: LocalKind::Temp,
                    span: Span::DUMMY,
                },
            ],
            blocks: vec![BasicBlock {
                stmts: vec![Stmt::new(
                    StmtKind::Assign {
                        place: Place::local(LocalId(0)),
                        rvalue,
                    },
                    span,
                )],
                terminator: Terminator::Return,
                terminator_span: span,
            }],
            arg_count: 0,
            param_modes: Vec::new(),
            span: Span::DUMMY,
            borrows: None,
            sources: Vec::new(),
            is_lambda: false,
            emit_if_used: false,
            borrowed_params: Vec::new(),
            call_argument_bindings: Vec::new(),
            for_iterators: Vec::new(),
            callable_regions: None,
            closure_environment: None,
            closure_captures_by_move: false,
            class_owner: None,
            class_virtual_slot: None,
            is_abstract: false,
            is_extern_declaration: false,
            ffi_counted: None,
            mut_self: false,
            elided_accesses: Vec::new(),
            hoisted_accesses: Vec::new(),
            uncounted_handles: Vec::new(),
            removed_checks: Vec::new(),
            restrict_views: false,
            stack_buffers: Vec::new(),
        };
        (body, types)
    }

    #[test]
    fn an_environment_built_from_its_captures_is_accepted() {
        let (body, types) = env_body(Rvalue::Aggregate {
            kind: crate::AggregateKind::Struct(ember_types::StructId(0)),
            operands: vec![Operand::Copy(Place::local(LocalId(1)))],
        });
        assert!(verify_views(&body, &types).is_empty());
    }

    #[test]
    fn an_environment_manufactured_by_a_cast_is_refused() {
        let (body, types) = env_body(Rvalue::Cast {
            kind: crate::CastKind::Numeric,
            operand: Operand::Copy(Place::local(LocalId(1))),
            to: body_ty_placeholder(),
        });
        let violations = verify_views(&body, &types);
        assert_eq!(violations.len(), 1, "got {violations:?}");
        assert!(violations[0].message.contains("carries no borrow"));
    }

    #[test]
    fn the_codegen_boundary_rejects_a_view_without_provenance() {
        let (body, types) = env_body(Rvalue::Cast {
            kind: crate::CastKind::Numeric,
            operand: Operand::Copy(Place::local(LocalId(1))),
            to: body_ty_placeholder(),
        });
        let bodies = vec![body];
        let Err(violations) = for_codegen(&bodies, &types) else {
            panic!("a provenance-free view crossed the codegen boundary");
        };
        assert!(
            violations
                .iter()
                .any(|v| v.message.contains("carries no borrow"))
        );
    }

    fn body_ty_placeholder() -> Ty {
        TypeTable::new().1.i32
    }
}

#[cfg(test)]
mod interface_upcast_invariant_tests {
    use super::*;
    use crate::{BasicBlock, InterfaceAdapterMethod, LocalDecl, LocalKind, Stmt};
    use ember_span::{Span, Symbol};
    use ember_types::{ClassDef, ClassOpenness, TyKind, TypeTable};

    fn upcast_body(
        class_metadata_matches_source: bool,
        receiver: ember_hir::Mode,
    ) -> (Body, TypeTable) {
        let (mut types, common) = TypeTable::new();
        let span = Span::new(ember_span::FileId(0), 0, 1);
        let interface = Symbol::intern("Render");
        let class = types.add_class(ClassDef {
            is_sync: false,
            name: Symbol::intern("Pixel"),
            fields: Vec::new(),
            span,
            openness: ClassOpenness::Final,
            base: None,
            has_drop: false,
            origin: None,
            declaring_module: 0,
        });
        let class_ty = types.intern(TyKind::Class(class));
        let source = types.intern(TyKind::Ref {
            mutable: false,
            inner: class_ty,
        });
        let erased = types.intern(TyKind::Dyn {
            interfaces: vec![interface],
        });
        let target = types.intern(TyKind::Ref {
            mutable: false,
            inner: erased,
        });
        let metadata = if class_metadata_matches_source {
            class
        } else {
            ember_types::ClassId(class.0 + 1)
        };
        let body = Body {
            name: "upcast".to_string(),
            symbol: ember_branding::mangled("upcast"),
            is_unsafe: false,
            abi: None,
            overflow: ember_types::OverflowPolicy::Panic,
            fp: ember_types::FpMode::Strict,
            inline: ember_hir::InlineHint::default(),
            export_thread_policy: ember_hir::ExportThreadPolicy::Any,
            locals: vec![
                LocalDecl {
                    ty: target,
                    name: None,
                    kind: LocalKind::Return,
                    span: Span::DUMMY,
                },
                LocalDecl {
                    ty: source,
                    name: None,
                    kind: LocalKind::Temp,
                    span: Span::DUMMY,
                },
            ],
            blocks: vec![BasicBlock {
                stmts: vec![Stmt::new(
                    StmtKind::Assign {
                        place: Place::local(LocalId(0)),
                        rvalue: Rvalue::Cast {
                            kind: crate::CastKind::InterfaceUpcast {
                                concrete: if class_metadata_matches_source {
                                    class_ty
                                } else {
                                    types.intern(TyKind::Class(metadata))
                                },
                                interfaces: vec![interface],
                                layout: vec![Some(ember_hir::InterfaceSlot {
                                    params: Vec::new(),
                                    ret: common.i32,
                                })],
                                implementations: vec![Some(InterfaceAdapterMethod {
                                    symbol: ember_branding::mangled("Pixel_render"),
                                    receiver,
                                })],
                            },
                            operand: Operand::Copy(Place::local(LocalId(1))),
                            to: target,
                        },
                    },
                    span,
                )],
                terminator: Terminator::Return,
                terminator_span: span,
            }],
            arg_count: 0,
            param_modes: Vec::new(),
            span: Span::DUMMY,
            borrows: None,
            sources: Vec::new(),
            is_lambda: false,
            emit_if_used: false,
            borrowed_params: Vec::new(),
            call_argument_bindings: Vec::new(),
            for_iterators: Vec::new(),
            callable_regions: None,
            closure_environment: None,
            closure_captures_by_move: false,
            class_owner: None,
            class_virtual_slot: None,
            is_abstract: false,
            is_extern_declaration: false,
            ffi_counted: None,
            mut_self: false,
            elided_accesses: Vec::new(),
            hoisted_accesses: Vec::new(),
            uncounted_handles: Vec::new(),
            removed_checks: Vec::new(),
            restrict_views: false,
            stack_buffers: Vec::new(),
        };
        (body, types)
    }

    #[test]
    fn checked_class_to_interface_upcast_is_accepted() {
        let (body, types) = upcast_body(true, ember_hir::Mode::Borrow);
        assert!(verify_interface_upcasts(&body, &types).is_empty());
    }

    #[test]
    fn upcast_with_class_metadata_not_matching_its_borrow_is_refused() {
        let (body, types) = upcast_body(false, ember_hir::Mode::Borrow);
        let violations = verify_interface_upcasts(&body, &types);
        assert!(
            violations
                .iter()
                .any(|violation| violation.message.contains("concrete metadata disagrees")),
            "missing class/source mismatch violation: {violations:?}"
        );
    }

    #[test]
    fn upcast_with_owned_receiver_is_refused() {
        let (body, types) = upcast_body(true, ember_hir::Mode::Owned);
        let violations = verify_interface_upcasts(&body, &types);
        assert!(
            violations
                .iter()
                .any(|violation| violation.message.contains("owned receiver")),
            "missing owned-receiver violation: {violations:?}"
        );
    }
}
