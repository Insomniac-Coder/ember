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
        for stmt in &block.stmts {
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
