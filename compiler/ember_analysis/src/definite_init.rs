//! Definite initialisation (Part XVIII §4.6).
//!
//! > Forward dataflow over MIR with a lattice per local
//! > (`Uninit | Init | Maybe`) … Reading `Uninit`/`Maybe` is `E3050`/`E2100`;
//! > `Maybe` at a drop point introduces a **drop flag**.
//!
//! Phase 1 implements the locals half. Per-field paths for partial moves, and
//! the `self`-fields form that `[CLS-2]` needs inside `init`, arrive with
//! structs-with-drop in Phase 2 and classes in Phase 3. Drop flags belong to
//! drop elaboration and are not emitted here.

use ember_diag::{Diagnostic, Sink, codes};
use ember_mir::{
    Body, LocalId, LocalKind, Operand, Place, Projection, Rvalue, StmtKind, Terminator,
};
use ember_span::Span;

use crate::facts::{InitializationFacts, InitializationState as State};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct InitializationFactViolation {
    pub body: String,
    pub message: String,
}

/// Check one body and report every read of a local that may not be
/// initialised. Returns the number of errors reported.
pub fn check_definite_init(body: &Body, sink: &mut Sink) -> usize {
    let facts = analyze_definite_init(body);
    let violations = verify_initialization_facts(body, &facts);
    assert!(
        violations.is_empty(),
        "initialization fact verification failed for `{}`: {violations:?}",
        body.symbol
    );
    check_definite_init_with_facts(body, &facts, sink)
}

/// Produce the shared `[IMP-7]` initialization facts for one initial MIR
/// body. Diagnostics and verification both consume this result; neither
/// performs a second fixpoint.
pub fn analyze_definite_init(body: &Body) -> InitializationFacts {
    let entry = initial_state(body);

    // Per-block entry states, refined until they stop changing. Bodies are
    // small and reducible, so a plain worklist converges quickly; there is no
    // need for the dominator machinery a larger analysis would want.
    let mut block_entry: Vec<Option<Vec<State>>> = vec![None; body.blocks.len()];
    if body.blocks.is_empty() {
        return InitializationFacts {
            body_symbol: body.symbol.clone(),
            block_entry,
            block_exit: Vec::new(),
        };
    }
    block_entry[0] = Some(entry);

    let mut worklist = vec![0usize];
    while let Some(index) = worklist.pop() {
        let Some(state) = block_entry[index].clone() else { continue };
        let exit = transfer(body, index, state, None);
        for successor in successors(body, index) {
            let merged = match &block_entry[successor] {
                Some(existing) => {
                    let joined: Vec<State> =
                        existing.iter().zip(&exit).map(|(a, b)| a.join(*b)).collect();
                    if joined == *existing {
                        continue;
                    }
                    joined
                }
                None => exit.clone(),
            };
            block_entry[successor] = Some(merged);
            worklist.push(successor);
        }
    }

    let block_exit = block_entry
        .iter()
        .enumerate()
        .map(|(index, entry)| entry.clone().map(|state| transfer(body, index, state, None)))
        .collect();

    InitializationFacts { body_symbol: body.symbol.clone(), block_entry, block_exit }
}

/// Diagnose reads using the already-solved canonical fact set.
pub fn check_definite_init_with_facts(
    body: &Body,
    facts: &InitializationFacts,
    sink: &mut Sink,
) -> usize {
    debug_assert_eq!(facts.body_symbol(), body.symbol);
    let mut reporter =
        Reporter { body, sink, errors: 0, reported: vec![false; body.locals.len()] };
    for index in 0..body.blocks.len() {
        if let Some(state) = facts.block_entry(index) {
            transfer(body, index, state.to_vec(), Some(&mut reporter));
        }
    }
    reporter.errors
}

/// Verify that a canonical initialization fact set still describes this MIR.
///
/// This is deliberately stronger than checking vector lengths. It proves the
/// entry seed, each transfer result, and every predecessor join. A fact set
/// whose safety result no longer matches the CFG therefore cannot cross the
/// analysis boundary as trusted metadata.
pub fn verify_initialization_facts(
    body: &Body,
    facts: &InitializationFacts,
) -> Vec<InitializationFactViolation> {
    let mut violations = Vec::new();
    let mut fail = |message: String| {
        violations.push(InitializationFactViolation { body: body.symbol.clone(), message });
    };

    if facts.body_symbol() != body.symbol {
        fail(format!(
            "fact set belongs to `{}`, not `{}`",
            facts.body_symbol(), body.symbol
        ));
        return violations;
    }
    if facts.block_entry.len() != body.blocks.len()
        || facts.block_exit.len() != body.blocks.len()
    {
        fail(format!(
            "fact set has {} entries and {} exits for {} MIR blocks",
            facts.block_entry.len(),
            facts.block_exit.len(),
            body.blocks.len()
        ));
        return violations;
    }
    if body.blocks.is_empty() {
        return violations;
    }

    let expected_seed = initial_state(body);

    for index in 0..body.blocks.len() {
        let Some(entry) = &facts.block_entry[index] else {
            if facts.block_exit[index].is_some() {
                fail(format!("bb{index} is unreachable but has exit facts"));
            }
            continue;
        };
        if entry.len() != body.locals.len() {
            fail(format!(
                "bb{index} has {} entry facts for {} locals",
                entry.len(),
                body.locals.len()
            ));
            continue;
        }
        let expected_exit = transfer(body, index, entry.clone(), None);
        if facts.block_exit[index].as_ref() != Some(&expected_exit) {
            fail(format!("bb{index} exit facts do not match its MIR transfer"));
        }
    }

    let mut expected_entries: Vec<Option<Vec<State>>> = vec![None; body.blocks.len()];
    expected_entries[0] = Some(expected_seed);
    for (predecessor, exit) in facts.block_exit.iter().enumerate() {
        let Some(exit) = exit else { continue };
        for successor in successors(body, predecessor) {
            let slot = &mut expected_entries[successor];
            *slot = Some(match slot.take() {
                Some(existing) => {
                    existing.iter().zip(exit).map(|(a, b)| a.join(*b)).collect()
                }
                None => exit.clone(),
            });
        }
    }
    for (index, expected) in expected_entries.iter().enumerate() {
        if &facts.block_entry[index] != expected {
            fail(format!("bb{index} entry facts do not equal the join of predecessor exits"));
        }
    }

    violations
}

struct Reporter<'a> {
    body: &'a Body,
    sink: &'a mut Sink,
    errors: usize,
    /// One diagnostic per local: a variable read before assignment in a loop
    /// would otherwise be reported at every use.
    reported: Vec<bool>,
}

impl Reporter<'_> {
    fn report(&mut self, local: LocalId, state: State, span: Span) {
        let index = local.0 as usize;
        if self.reported[index] {
            return;
        }
        self.reported[index] = true;
        self.errors += 1;

        let decl = self.body.local(local);
        let name = decl.name.clone().unwrap_or_else(|| format!("_{}", local.0));
        let message = match state {
            State::Uninit => format!("`{name}` is used before it is given a value"),
            _ => format!("`{name}` may not have a value here"),
        };
        let mut diagnostic = Diagnostic::error(codes::E3050, span, message)
            .primary_label("used here");
        if !decl.span.is_dummy() {
            diagnostic = diagnostic.secondary(decl.span, "declared without a value here");
        }
        if state == State::Maybe {
            diagnostic = diagnostic
                .note("some paths reaching this point assign it and others do not")
                .help("give it a value in every branch, or assign one at the declaration");
        } else {
            diagnostic = diagnostic.help("assign a value before reading it");
        }
        self.sink.emit(diagnostic);
    }
}

/// Parameters arrive initialised; every other local starts empty.
fn initial_state(body: &Body) -> Vec<State> {
    body.locals
        .iter()
        .map(|decl| match decl.kind {
            LocalKind::Arg => State::Init,
            // The return slot is written before `Return` on any path that
            // returns a value, and a void function never writes it at all.
            LocalKind::Return => State::Init,
            _ => State::Uninit,
        })
        .collect()
}

/// Walk one block, updating `state`. With a reporter, also diagnose reads.
fn transfer(
    body: &Body,
    index: usize,
    mut state: Vec<State>,
    mut reporter: Option<&mut Reporter>,
) -> Vec<State> {
    let block = &body.blocks[index];

    for stmt in &block.stmts {
        match &stmt.kind {
            StmtKind::Assign { place, rvalue } => {
                read_rvalue(rvalue, &state, stmt.span, &mut reporter);
                // Assigning through a projection reads the base first: you
                // cannot write `x.field` without having an `x`.
                if !place.projection.is_empty() {
                    read_place(place, &state, stmt.span, &mut reporter);
                }
                write_place(place, &mut state);
            }
            StmtKind::CheckedBinaryOp { dest, overflow, lhs, rhs, .. } => {
                read_operand(lhs, &state, stmt.span, &mut reporter);
                read_operand(rhs, &state, stmt.span, &mut reporter);
                write_place(dest, &mut state);
                write_place(overflow, &mut state);
            }
            // `StorageLive` brings a slot into scope; it does not give it a
            // value. Re-entering a loop must clear it, or a value assigned on
            // the previous iteration would look live on the next.
            StmtKind::StorageLive(local) | StmtKind::StorageDead(local) => {
                state[local.0 as usize] = State::Uninit;
            }
            // A drop reads the place it is dropping.
            StmtKind::Drop { place, .. } => {
                read_place(place, &state, stmt.span, &mut reporter);
            }
            StmtKind::Nop => {}
        }
    }

    let span = block.terminator_span;
    match &block.terminator {
        Terminator::SwitchInt { discr, .. } => read_operand(discr, &state, span, &mut reporter),
        Terminator::Assert { cond, .. } => read_operand(cond, &state, span, &mut reporter),
        Terminator::Call { args, dest, .. } => {
            for arg in args {
                read_operand(arg, &state, span, &mut reporter);
            }
            write_place(dest, &mut state);
        }
        Terminator::Goto(_) | Terminator::Return | Terminator::Unreachable => {}
    }

    state
}

fn successors(body: &Body, index: usize) -> Vec<usize> {
    match &body.blocks[index].terminator {
        Terminator::Goto(bb) => vec![bb.0 as usize],
        Terminator::Call { next, .. } | Terminator::Assert { next, .. } => vec![next.0 as usize],
        Terminator::SwitchInt { targets, otherwise, .. } => {
            let mut out: Vec<usize> = targets.iter().map(|(_, bb)| bb.0 as usize).collect();
            out.push(otherwise.0 as usize);
            out
        }
        Terminator::Return | Terminator::Unreachable => Vec::new(),
    }
}

fn write_place(place: &Place, state: &mut [State]) {
    // A write through a projection leaves the whole local initialised only if
    // it already was; Phase 2's per-field paths make this precise.
    if place.projection.is_empty() {
        state[place.local.0 as usize] = State::Init;
    }
}

fn read_place(place: &Place, state: &[State], span: Span, reporter: &mut Option<&mut Reporter>) {
    // `a[i]` reads `i` as well as `a`. The lowering only ever puts a freshly
    // assigned temporary there today, but the projection is a read either way
    // and the analysis should not depend on who built it.
    for projection in &place.projection {
        if let Projection::Index(local) = projection {
            let current = state[local.0 as usize];
            if !current.is_readable() {
                if let Some(reporter) = reporter {
                    reporter.report(*local, current, span);
                }
            }
        }
    }
    let current = state[place.local.0 as usize];
    if current.is_readable() {
        return;
    }
    if let Some(reporter) = reporter {
        reporter.report(place.local, current, span);
    }
}

fn read_operand(operand: &Operand, state: &[State], span: Span, reporter: &mut Option<&mut Reporter>) {
    match operand {
        Operand::Copy(place) | Operand::Move(place) => read_place(place, state, span, reporter),
        Operand::Const(_) => {}
    }
}

fn read_rvalue(rvalue: &Rvalue, state: &[State], span: Span, reporter: &mut Option<&mut Reporter>) {
    match rvalue {
        Rvalue::Use(o) | Rvalue::UnaryOp { operand: o, .. } => {
            read_operand(o, state, span, reporter)
        }
        Rvalue::Cast { operand, .. } => read_operand(operand, state, span, reporter),
        Rvalue::BinaryOp { lhs, rhs, .. } => {
            read_operand(lhs, state, span, reporter);
            read_operand(rhs, state, span, reporter);
        }
        Rvalue::Aggregate { operands, .. } => {
            for operand in operands {
                read_operand(operand, state, span, reporter);
            }
        }
        Rvalue::Repeat { value, .. } => read_operand(value, state, span, reporter),
        Rvalue::Discriminant(place) => read_place(place, state, span, reporter),
        // Taking a reference reads the place's address, not its value, but a
        // `mut` argument must still name something that exists.
        Rvalue::Ref { place, .. } => read_place(place, state, span, reporter),
    }
}

/// Produce canonical initialization facts for every initial MIR body.
pub fn analyze_all(bodies: &[Body]) -> Vec<InitializationFacts> {
    bodies.iter().map(analyze_definite_init).collect()
}

/// Diagnose every body from a previously verified canonical fact set.
pub fn check_all_with_facts(
    bodies: &[Body],
    facts: &[InitializationFacts],
    sink: &mut Sink,
) -> usize {
    assert_eq!(
        bodies.len(),
        facts.len(),
        "initialization fact count does not match the MIR body count"
    );
    bodies
        .iter()
        .zip(facts)
        .map(|(body, facts)| check_definite_init_with_facts(body, facts, sink))
        .sum()
}

/// Compatibility entry point for callers that do not need to retain facts.
/// The explicit driver pipeline uses `analyze_all`, verifies, then calls
/// `check_all_with_facts` so the phase boundary remains visible.
pub fn check_all(bodies: &[Body], sink: &mut Sink) -> usize {
    let facts = analyze_all(bodies);
    verify_initialization_facts_all(bodies, &facts);
    check_all_with_facts(bodies, &facts, sink)
}

/// Verify every body's initialization facts, panicking on a compiler-internal
/// mismatch. User source errors are diagnosed by the consumer above; a fact
/// mismatch means the compiler's trusted semantic boundary is stale.
pub fn verify_initialization_facts_all(bodies: &[Body], facts: &[InitializationFacts]) {
    assert_eq!(
        bodies.len(),
        facts.len(),
        "initialization fact count does not match the MIR body count"
    );
    let violations: Vec<InitializationFactViolation> = bodies
        .iter()
        .zip(facts)
        .flat_map(|(body, facts)| verify_initialization_facts(body, facts))
        .collect();
    assert!(
        violations.is_empty(),
        "initialization fact verification failed:\n{}",
        violations
            .iter()
            .map(|v| format!("  {}: {}", v.body, v.message))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use ember_mir::{BasicBlock, BasicBlockId, Const, LocalDecl, Stmt};

    fn branch_body() -> Body {
        let (_, common) = ember_types::TypeTable::new();
        let span = Span::new(ember_span::FileId(0), 0, 1);
        Body {
            name: "facts".to_string(),
            symbol: "facts".to_string(),
            locals: vec![
                LocalDecl {
                    ty: common.void,
                    kind: LocalKind::Return,
                    name: None,
                    span,
                },
                LocalDecl {
                    ty: common.bool_,
                    kind: LocalKind::Arg,
                    name: Some("choose".to_string()),
                    span,
                },
                LocalDecl {
                    ty: common.i32,
                    kind: LocalKind::User,
                    name: Some("value".to_string()),
                    span,
                },
            ],
            blocks: vec![
                BasicBlock {
                    stmts: vec![Stmt::new(StmtKind::StorageLive(LocalId(2)), Span::DUMMY)],
                    terminator: Terminator::SwitchInt {
                        discr: Operand::Copy(Place::local(LocalId(1))),
                        targets: vec![(1, BasicBlockId(1))],
                        otherwise: BasicBlockId(2),
                    },
                    terminator_span: span,
                },
                BasicBlock {
                    stmts: vec![Stmt::new(
                        StmtKind::Assign {
                            place: Place::local(LocalId(2)),
                            rvalue: Rvalue::Use(Operand::Const(Const::Int {
                                value: 1,
                                ty: common.i32,
                            })),
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
            ],
            arg_count: 1,
            span,
            borrows: None,
            borrowed_params: Vec::new(),
            for_iterators: Vec::new(),
        }
    }

    #[test]
    fn the_join_is_a_lattice() {
        assert_eq!(State::Init.join(State::Init), State::Init);
        assert_eq!(State::Uninit.join(State::Uninit), State::Uninit);
        assert_eq!(State::Init.join(State::Uninit), State::Maybe);
        assert_eq!(State::Uninit.join(State::Init), State::Maybe);
        assert_eq!(State::Maybe.join(State::Init), State::Maybe);
    }

    #[test]
    fn only_init_is_readable() {
        assert!(State::Init.is_readable());
        assert!(!State::Uninit.is_readable());
        assert!(!State::Maybe.is_readable());
    }

    #[test]
    fn canonical_facts_retain_the_branch_join() {
        let body = branch_body();
        let facts = analyze_definite_init(&body);
        assert_eq!(facts.block_count(), body.blocks.len());
        assert_eq!(facts.block_entry(0), Some(&[State::Init, State::Init, State::Uninit][..]));
        assert_eq!(facts.block_entry(3), Some(&[State::Init, State::Init, State::Maybe][..]));
        assert!(verify_initialization_facts(&body, &facts).is_empty());
    }

    #[test]
    fn the_fact_verifier_rejects_a_stale_transfer_result() {
        let body = branch_body();
        let mut facts = analyze_definite_init(&body);
        facts.block_exit[1].as_mut().unwrap()[2] = State::Uninit;
        let violations = verify_initialization_facts(&body, &facts);
        assert!(
            violations.iter().any(|v| v.message.contains("exit facts")),
            "expected stale-exit violation, got {violations:?}"
        );
    }

    #[test]
    fn the_fact_verifier_rejects_a_wrong_predecessor_join() {
        let body = branch_body();
        let mut facts = analyze_definite_init(&body);
        facts.block_entry[3].as_mut().unwrap()[2] = State::Init;
        let violations = verify_initialization_facts(&body, &facts);
        assert!(
            violations.iter().any(|v| v.message.contains("predecessor exits")),
            "expected predecessor-join violation, got {violations:?}"
        );
    }
}
