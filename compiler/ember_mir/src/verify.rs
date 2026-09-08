//! The MIR verifier (Part XVIII §4.2).
//!
//! Runs after every pass in debug builds of the compiler. It checks structural
//! invariants only — well-formedness of the CFG and of places — not semantics.
//! `[MIR-4]`: every block ends with exactly one terminator, which the type
//! system already guarantees, so the check here is that every terminator names
//! a block that exists.
//!
//! The invariants that need type information (`[MIR-1]` well-typed places,
//! `[MIR-2]` at most one move per path, `[MIR-3]` access brackets,
//! `[MIR-5]` retain/release only on handles) are added by the phases that
//! introduce the constructs they govern.

use crate::{BasicBlockId, Body, LocalId, Operand, Place, Rvalue, StmtKind, Terminator};

#[derive(Debug)]
pub struct Violation {
    pub body: String,
    pub message: String,
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
        self.violations.push(Violation { body: self.symbol.to_string(), message });
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
            self.fail(format!("{at} jumps to bb{}, which does not exist", target.0));
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
                StmtKind::CheckedBinaryOp { dest, overflow, lhs, rhs, .. } => {
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
            Terminator::SwitchInt { discr, targets, otherwise } => {
                v.operand(discr, &at);
                for (_, bb) in targets {
                    v.target(*bb, &at);
                }
                v.target(*otherwise, &at);
            }
            Terminator::Call { args, dest, next, .. } => {
                for a in args {
                    v.operand(a, &at);
                }
                v.place(dest, &at);
                v.target(*next, &at);
            }
            Terminator::Assert { cond, msg, next, .. } => {
                v.operand(cond, &at);
                if let crate::AssertKind::Bounds { len, index } = msg {
                    v.operand(len, &at);
                    v.operand(index, &at);
                }
                v.target(*next, &at);
            }
            Terminator::Return | Terminator::Unreachable => {}
        }
    }

    v.violations
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BasicBlock, LocalDecl, LocalKind, Stmt};
    use ember_span::Span;

    /// A body with one empty block and a real span on its terminator.
    fn body_with(stmts: Vec<Stmt>, terminator_span: Span) -> Body {
        Body {
            name: "t".to_string(),
            symbol: ember_branding::mangled("t"),
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
            span: Span::DUMMY,
            borrows: None,
        }
    }

    /// `[CG-C-8]` — the check must actually fire. A verifier check nobody can
    /// make fail is a comment.
    #[test]
    fn a_statement_without_a_span_is_a_violation() {
        let stmt = Stmt::new(StmtKind::Drop { place: Place::local(LocalId(0)), flag: None }, Span::DUMMY);
        let violations = verify(&body_with(vec![stmt], Span::new(ember_span::FileId(0), 0, 1)));
        assert!(
            violations.iter().any(|v| v.message.contains("no source span")),
            "expected a missing-span violation, got {violations:?}"
        );
    }

    #[test]
    fn a_terminator_without_a_span_is_a_violation() {
        let violations = verify(&body_with(Vec::new(), Span::DUMMY));
        assert!(
            violations.iter().any(|v| v.message.contains("terminator has no source span")),
            "expected a missing-span violation, got {violations:?}"
        );
    }

    #[test]
    fn bookkeeping_statements_need_no_span() {
        let stmt = Stmt::new(StmtKind::StorageLive(LocalId(0)), Span::DUMMY);
        let violations = verify(&body_with(vec![stmt], Span::new(ember_span::FileId(0), 0, 1)));
        assert!(violations.is_empty(), "got {violations:?}");
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
