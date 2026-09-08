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

/// Where a local stands on one path.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum State {
    Uninit,
    Init,
    /// Initialised on some paths reaching this point and not on others.
    Maybe,
}

impl State {
    /// The lattice join at a control-flow merge.
    fn join(self, other: State) -> State {
        match (self, other) {
            (State::Init, State::Init) => State::Init,
            (State::Uninit, State::Uninit) => State::Uninit,
            _ => State::Maybe,
        }
    }

    fn is_readable(self) -> bool {
        self == State::Init
    }
}

/// Check one body and report every read of a local that may not be
/// initialised. Returns the number of errors reported.
pub fn check_definite_init(body: &Body, sink: &mut Sink) -> usize {
    let local_count = body.locals.len();
    let entry = initial_state(body);

    // Per-block entry states, refined until they stop changing. Bodies are
    // small and reducible, so a plain worklist converges quickly; there is no
    // need for the dominator machinery a larger analysis would want.
    let mut block_entry: Vec<Option<Vec<State>>> = vec![None; body.blocks.len()];
    if body.blocks.is_empty() {
        return 0;
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

    // A second pass reports, so that a diagnostic is only produced once the
    // entry states have settled. Reporting during the fixpoint would emit the
    // same error on every iteration.
    let mut reporter = Reporter { body, sink, errors: 0, reported: vec![false; local_count] };
    for index in 0..body.blocks.len() {
        if let Some(state) = block_entry[index].clone() {
            transfer(body, index, state, Some(&mut reporter));
        }
    }
    reporter.errors
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

/// Run the analysis over every body.
pub fn check_all(bodies: &[Body], sink: &mut Sink) -> usize {
    bodies.iter().map(|body| check_definite_init(body, sink)).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
