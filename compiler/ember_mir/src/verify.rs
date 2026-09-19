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

use std::collections::VecDeque;

use crate::{
    BasicBlockId, Body, Builtin, Const, FuncRef, LocalId, Operand, Place, Projection, Rvalue,
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
        let FuncRef::Interface { slot, params, ret, layout, .. } = func else {
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
    }

    /// Verify `[MIR-3]` for the dynamic exclusivity brackets introduced by
    /// Phase 3 lowering.
    ///
    /// Access state is a small stack because nested reborrows are legal, but
    /// an `EndAccess` must close the most recently opened matching access.
    /// The state is propagated through the reachable CFG; a join is valid only
    /// when every predecessor arrives with the same stack.  This catches both
    /// a missing close on one branch and an interval that is closed on only
    /// some paths before a later merge.  It intentionally does not decide
    /// whether an access *may* be elided: that requires `[EXC-3a]`'s safety
    /// side-table producer and reporting consumer, which are a separate
    /// implementation boundary.
    fn access_intervals(&mut self, body: &Body) {
        type Access = (Place, bool);

        let mut incoming: Vec<Option<Vec<Access>>> = vec![None; body.blocks.len()];
        let mut work = VecDeque::from([(BasicBlockId(0), Vec::<Access>::new())]);

        while let Some((block_id, mut state)) = work.pop_front() {
            let index = block_id.0 as usize;
            if index >= body.blocks.len() {
                // The structural pass above emits the precise bad-target
                // diagnostic. Do not duplicate it here.
                continue;
            }

            if let Some(previous) = &incoming[index] {
                if previous != &state {
                    self.fail(format!("bb{index} receives incompatible dynamic-access stacks: previous {previous:?}, incoming {state:?}"));
                }
                continue;
            }
            incoming[index] = Some(state.clone());

            for stmt in &body.blocks[index].stmts {
                match &stmt.kind {
                    StmtKind::BeginAccess { place, mutable } => {
                        state.push((place.clone(), *mutable));
                    }
                    StmtKind::EndAccess { place, mutable } => {
                        let expected = (place.clone(), *mutable);
                        match state.last() {
                            Some(actual) if actual == &expected => {
                                state.pop();
                            }
                            Some(actual) => self.fail(format!("bb{index} ends dynamic access {expected:?}, but the innermost open access is {actual:?}")),
                            None => self.fail(format!("bb{index} ends dynamic access {expected:?}, but no dynamic access is open")),
                        }
                    }
                    _ => {}
                }
            }

            let mut enqueue = |target: BasicBlockId| {
                if (target.0 as usize) < body.blocks.len() {
                    work.push_back((target, state.clone()));
                }
            };
            match &body.blocks[index].terminator {
                Terminator::Goto(target) => enqueue(*target),
                Terminator::SwitchInt {
                    targets, otherwise, ..
                } => {
                    for (_, target) in targets {
                        enqueue(*target);
                    }
                    enqueue(*otherwise);
                }
                Terminator::Call { next, .. } | Terminator::Assert { next, .. } => enqueue(*next),
                Terminator::Return | Terminator::Unreachable => {
                    if !state.is_empty() {
                        self.fail(format!(
                            "bb{index} terminates with dynamic accesses still open: {state:?}"
                        ));
                    }
                }
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
                StmtKind::BeginAccess { place, .. } | StmtKind::EndAccess { place, .. } => {
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
                StmtKind::Drop { place, flag } => {
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
                v.target(*next, &at);
            }
            Terminator::Return | Terminator::Unreachable => {}
        }
    }

    // Run after structural place/target checks so the dataflow does not
    // produce duplicate diagnostics for malformed block references.
    v.access_intervals(body);

    v.violations
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BasicBlock, LocalDecl, LocalKind, ParameterMode, Stmt};
    use ember_span::Span;

    /// A body with one empty block and a real span on its terminator.
    fn body_with(stmts: Vec<Stmt>, terminator_span: Span) -> Body {
        Body {
            name: "t".to_string(),
            symbol: ember_branding::mangled("t"),
            is_unsafe: false,
            abi: None,
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
            borrowed_params: Vec::new(),
            for_iterators: Vec::new(),
            callable_regions: None,
            closure_environment: None,
            closure_captures_by_move: false,
            class_owner: None,
            class_virtual_slot: None,
            elided_accesses: Vec::new(),
        }
    }

    /// `[CG-C-8]` — the check must actually fire. A verifier check nobody can
    /// make fail is a comment.
    #[test]
    fn a_statement_without_a_span_is_a_violation() {
        let stmt = Stmt::new(
            StmtKind::Drop {
                place: Place::local(LocalId(0)),
                flag: None,
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
    fn dynamic_access_end_must_match_the_innermost_open_access() {
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
            violations
                .iter()
                .any(|violation| violation.message.contains("innermost open access")),
            "missing LIFO access violation: {violations:?}"
        );
    }

    #[test]
    fn dynamic_access_state_must_agree_at_cfg_joins() {
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
            violations.iter().any(|violation| violation
                .message
                .contains("incompatible dynamic-access stacks")),
            "missing CFG-join access violation: {violations:?}"
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
                interface: ember_span::Symbol::intern("Drawable"),
                slot: 1,
                params: Vec::new(),
                ret: common.i32,
                layout: vec![Some(ember_hir::InterfaceSlot {
                    params: Vec::new(),
                    ret: common.i32,
                })],
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
                Rvalue::Cast { kind: crate::CastKind::ClassUpcastBorrowed, .. } => None,
                Rvalue::Cast { .. } => Some("a cast"),
                Rvalue::BinaryOp { .. } => Some("an arithmetic operation"),
                Rvalue::UnaryOp { .. } => Some("a unary operation"),
                Rvalue::Discriminant(_) => Some("an enum discriminant"),
                Rvalue::Use(Operand::Const(c)) if !matches!(c, Const::Str(_)) => {
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
                    | Builtin::ArenaArrayWithCapacity { .. }
                    | Builtin::ArenaMapWithCapacity { .. }
                    | Builtin::ArenaAlloc { .. }
                    | Builtin::ArenaAllocUninit { .. }
                    | Builtin::ArenaAllocArrayZeroed { .. }
                    | Builtin::ArenaAllocArrayDefault { .. }
                    | Builtin::FixedArenaAlloc { .. }
                    | Builtin::ScopedArenaAlloc { .. }
                    | Builtin::ArenaScope { .. }
                    | Builtin::ArraySplitAtMut { .. }
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
                TyKind::Array { elem, .. } | TyKind::Vec { elem } | TyKind::Span { elem, .. },
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
                has_default: false,
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
            borrowed_params: Vec::new(),
            for_iterators: Vec::new(),
            callable_regions: None,
            closure_environment: None,
            closure_captures_by_move: false,
            class_owner: None,
            class_virtual_slot: None,
            elided_accesses: Vec::new(),
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
