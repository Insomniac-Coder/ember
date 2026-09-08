//! Moves and drop elaboration (Part XVIII §4.9, `[OWN-2]`, `[OWN-3]`,
//! `[DRP-2]`).
//!
//! A value's owner drops it when the owner's scope ends — unless the value was
//! moved out first, in which case the new owner drops it and the old one must
//! not. Which of the two applies is not always known at compile time:
//!
//! ```ember
//! xs = Array()
//! if c:
//!     consume(xs)     # moved on this path
//! # dropped here, but only if `c` was false
//! ```
//!
//! `[OWN-3]` settles that with a **drop flag**: a hidden `bool` per local that
//! is conditionally moved, set when the value is put there and cleared when it
//! is moved out. A conditional move is never an error.
//!
//! This pass runs after lowering, over one body:
//!
//! 1. a forward dataflow computing `Live | Moved | Maybe` per local;
//! 2. every use of a `Moved` local reported as `E3040`;
//! 3. every `Drop` of a `Moved` local removed, and every `Drop` of a `Maybe`
//!    local given a flag;
//! 4. the flags' assignments written in beside the moves.

use std::collections::BTreeMap;

use ember_diag::{Diagnostic, Sink, codes};
use ember_mir::{
    Body, LocalDecl, LocalId, LocalKind, Operand, Place, Rvalue, Stmt, StmtKind, Terminator,
};
use ember_span::Span;
use ember_types::{Ty, TypeTable};

/// Where a local stands on one path.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum Owned {
    /// The value is here and this local will drop it.
    Live,
    /// It was moved out; someone else drops it.
    Moved,
    /// Moved on some paths reaching this point and not on others.
    Maybe,
}

impl Owned {
    fn join(self, other: Owned) -> Owned {
        if self == other { self } else { Owned::Maybe }
    }
}

/// Rewrite one body so that every value is dropped exactly once, and report
/// every use of a moved value. Returns the number of errors.
pub fn elaborate(body: &mut Body, types: &TypeTable, sink: &mut Sink) -> usize {
    let entry = initial(body);
    let mut block_entry: Vec<Option<Vec<Owned>>> = vec![None; body.blocks.len()];
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
                    let joined: Vec<Owned> =
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

    // A second pass reports and rewrites, once the entry states have settled.
    let mut reporter = Reporter { sink, errors: 0, reported: vec![false; body.locals.len()] };
    let mut plan: Vec<(usize, usize, Owned)> = Vec::new();
    for (index, entry) in block_entry.iter().enumerate() {
        if let Some(state) = entry.clone() {
            let mut state = state;
            for (position, stmt) in body.blocks[index].stmts.iter().enumerate() {
                if let StmtKind::Drop { place, .. } = &stmt.kind {
                    if place.projection.is_empty() {
                        plan.push((index, position, state[place.local.0 as usize]));
                    }
                }
                step(stmt, &mut state, Some(&mut reporter), body);
            }
            step_terminator(body, index, &mut state, Some(&mut reporter));
        }
    }
    let errors = reporter.errors;

    // `[OWN-3]` — a local that is `Maybe` at any drop needs a flag.
    let needs_flag: Vec<LocalId> = plan
        .iter()
        .filter(|(_, _, owned)| *owned == Owned::Maybe)
        .filter_map(|(block, position, _)| match &body.blocks[*block].stmts[*position].kind {
            StmtKind::Drop { place, .. } => Some(place.local),
            _ => None,
        })
        .collect();
    let flags = create_flags(body, &needs_flag, types);

    // Apply: drop of a moved value goes away, and a `Maybe` drop gets its flag.
    for (block, position, owned) in plan {
        let stmt = &mut body.blocks[block].stmts[position];
        let StmtKind::Drop { place, flag } = &mut stmt.kind else { continue };
        match owned {
            Owned::Live => {}
            Owned::Moved => stmt.kind = StmtKind::Nop,
            Owned::Maybe => *flag = flags.get(&place.local).copied(),
        }
    }
    if !flags.is_empty() {
        write_flag_updates(body, &flags);
    }
    errors
}

/// A `bool` local per conditionally moved value, cleared at the start of the
/// body so that a path which never assigned the value does not drop it.
fn create_flags(
    body: &mut Body,
    wanted: &[LocalId],
    types: &TypeTable,
) -> BTreeMap<LocalId, LocalId> {
    let mut flags = BTreeMap::new();
    let bool_ty = find_bool(types).unwrap_or_else(|| body.locals[0].ty);
    for local in wanted {
        if flags.contains_key(local) {
            continue;
        }
        let flag = LocalId(body.locals.len() as u32);
        let span = body.local(*local).span;
        body.locals.push(LocalDecl {
            ty: bool_ty,
            kind: LocalKind::Temp,
            name: body.local(*local).name.as_ref().map(|n| format!("{n}_live")),
            span,
        });
        flags.insert(*local, flag);
    }
    flags
}

/// Set each flag where its value is stored, clear it where the value is moved
/// out, and start every flag false.
fn write_flag_updates(body: &mut Body, flags: &BTreeMap<LocalId, LocalId>) {
    for block in body.blocks.iter_mut() {
        let mut rewritten: Vec<Stmt> = Vec::with_capacity(block.stmts.len());
        for stmt in std::mem::take(&mut block.stmts) {
            let span = stmt.span;
            // A move out of a flagged local clears its flag.
            let mut cleared: Vec<LocalId> = Vec::new();
            if let StmtKind::Assign { rvalue, .. } = &stmt.kind {
                moved_by_rvalue(rvalue, &mut cleared);
            }
            // Storing into a flagged local sets its flag.
            let set = match &stmt.kind {
                StmtKind::Assign { place, .. } if place.projection.is_empty() => {
                    flags.get(&place.local).copied()
                }
                StmtKind::CheckedBinaryOp { dest, .. } if dest.projection.is_empty() => {
                    flags.get(&dest.local).copied()
                }
                _ => None,
            };
            rewritten.push(stmt);
            for local in cleared {
                if let Some(&flag) = flags.get(&local) {
                    rewritten.push(assign_bool(flag, false, span));
                }
            }
            if let Some(flag) = set {
                rewritten.push(assign_bool(flag, true, span));
            }
        }
        block.stmts = rewritten;
    }
    // A value can also arrive from a call, whose destination is written by a
    // terminator rather than by a statement — `xs = Array()` is one. The flag
    // is set at the top of the block control reaches next.
    let mut set_on_entry: Vec<(usize, LocalId, Span)> = Vec::new();
    for (index, block) in body.blocks.iter().enumerate() {
        if let Terminator::Call { dest, next, .. } = &block.terminator {
            if dest.projection.is_empty() {
                if let Some(&flag) = flags.get(&dest.local) {
                    set_on_entry.push((next.0 as usize, flag, block.terminator_span));
                }
            }
        }
        let _ = index;
    }
    for (block, flag, span) in set_on_entry {
        let mut stmts = vec![assign_bool(flag, true, span)];
        stmts.extend(std::mem::take(&mut body.blocks[block].stmts));
        body.blocks[block].stmts = stmts;
    }

    // Every flag starts false: the value is not there until it is stored.
    let span = body.span;
    let mut prologue: Vec<Stmt> = flags
        .values()
        .map(|flag| assign_bool(*flag, false, span))
        .collect();
    prologue.extend(std::mem::take(&mut body.blocks[0].stmts));
    body.blocks[0].stmts = prologue;
}

fn assign_bool(flag: LocalId, value: bool, span: Span) -> Stmt {
    Stmt::new(
        StmtKind::Assign {
            place: Place::local(flag),
            rvalue: Rvalue::Use(Operand::Const(ember_mir::Const::Bool(value))),
        },
        span,
    )
}

fn find_bool(types: &TypeTable) -> Option<Ty> {
    types
        .all()
        .find(|(_, kind)| matches!(kind, ember_types::TyKind::Bool))
        .map(|(ty, _)| ty)
}

/// Parameters and the return slot arrive owned; every other local starts
/// empty, which for this analysis is the same as moved-out.
fn initial(body: &Body) -> Vec<Owned> {
    body.locals
        .iter()
        .map(|decl| match decl.kind {
            LocalKind::Arg | LocalKind::Return => Owned::Live,
            _ => Owned::Moved,
        })
        .collect()
}

fn transfer(
    body: &Body,
    index: usize,
    mut state: Vec<Owned>,
    mut reporter: Option<&mut Reporter>,
) -> Vec<Owned> {
    for stmt in &body.blocks[index].stmts {
        step(stmt, &mut state, reporter.as_deref_mut(), body);
    }
    step_terminator(body, index, &mut state, reporter);
    state
}

/// A call's arguments are read by the terminator, not by a statement, so a use
/// of a moved value in an argument is only visible here. Both the fixpoint and
/// the reporting pass go through this, or the two would disagree about what
/// each block does.
fn step_terminator(
    body: &Body,
    index: usize,
    state: &mut [Owned],
    reporter: Option<&mut Reporter>,
) {
    let span = body.blocks[index].terminator_span;
    let Terminator::Call { args, dest, .. } = &body.blocks[index].terminator else { return };

    let mut read = Vec::new();
    let mut push = |place: &Place| read.push(place.local);
    for arg in args {
        read_by_operand(arg, &mut push);
    }
    if let Some(reporter) = reporter {
        for local in &read {
            if state[local.0 as usize] != Owned::Live {
                reporter.report(*local, state[local.0 as usize], span, body);
            }
        }
    }
    let mut moved = Vec::new();
    for arg in args {
        moved_by_operand(arg, &mut moved);
    }
    for local in moved {
        state[local.0 as usize] = Owned::Moved;
    }
    if dest.projection.is_empty() {
        state[dest.local.0 as usize] = Owned::Live;
    }
}

fn step(stmt: &Stmt, state: &mut [Owned], reporter: Option<&mut Reporter>, body: &Body) {
    match &stmt.kind {
        StmtKind::Assign { place, rvalue } => {
            // `[OWN-3]` — reading a moved value is `E3040`.
            let mut read = Vec::new();
            read_by_rvalue(rvalue, &mut read);
            // `[BRW-7]` — a borrow does not consume, so it is not in `read`,
            // but a reference into memory that was moved out of dangles.
            let borrowed = match rvalue {
                Rvalue::Ref { place, .. } => Some(place.local),
                _ => None,
            };
            if let Some(reporter) = reporter {
                for local in &read {
                    if state[local.0 as usize] != Owned::Live {
                        reporter.report(*local, state[local.0 as usize], stmt.span, body);
                    }
                }
                if let Some(local) = borrowed {
                    if state[local.0 as usize] != Owned::Live {
                        reporter.report_borrow(local, state[local.0 as usize], stmt.span, body);
                    }
                }
            }
            let mut moved = Vec::new();
            moved_by_rvalue(rvalue, &mut moved);
            for local in moved {
                state[local.0 as usize] = Owned::Moved;
            }
            if place.projection.is_empty() {
                state[place.local.0 as usize] = Owned::Live;
            }
        }
        // `[TYP-8]` — checked arithmetic writes its result through its own
        // statement rather than an `Assign`, so without this arm the
        // destination was never marked live and the next read of it was
        // reported as a use after move. Every integer `a * b` bound to a
        // local went through here.
        StmtKind::CheckedBinaryOp { dest, overflow, lhs, rhs, .. } => {
            let mut read = Vec::new();
            let mut push = |place: &Place| read.push(place.local);
            read_by_operand(lhs, &mut push);
            read_by_operand(rhs, &mut push);
            if let Some(reporter) = reporter {
                for local in &read {
                    if state[local.0 as usize] != Owned::Live {
                        reporter.report(*local, state[local.0 as usize], stmt.span, body);
                    }
                }
            }
            let mut moved = Vec::new();
            moved_by_operand(lhs, &mut moved);
            moved_by_operand(rhs, &mut moved);
            for local in moved {
                state[local.0 as usize] = Owned::Moved;
            }
            for place in [dest, overflow] {
                if place.projection.is_empty() {
                    state[place.local.0 as usize] = Owned::Live;
                }
            }
        }
        // A drop ends the value's life whatever it was.
        StmtKind::Drop { place, .. } => {
            if place.projection.is_empty() {
                state[place.local.0 as usize] = Owned::Moved;
            }
        }
        StmtKind::StorageLive(local) => state[local.0 as usize] = Owned::Moved,
        _ => {}
    }
}

/// Which locals an rvalue moves out of. Only a whole-local `Move` counts: a
/// move out of a field is a partial move, which Phase 2's later work handles
/// with per-field paths.
fn moved_by_rvalue(rvalue: &Rvalue, out: &mut Vec<LocalId>) {
    match rvalue {
        Rvalue::Use(o) | Rvalue::UnaryOp { operand: o, .. } => moved_by_operand(o, out),
        Rvalue::Cast { operand, .. } => moved_by_operand(operand, out),
        Rvalue::BinaryOp { lhs, rhs, .. } => {
            moved_by_operand(lhs, out);
            moved_by_operand(rhs, out);
        }
        Rvalue::Aggregate { operands, .. } => {
            operands.iter().for_each(|o| moved_by_operand(o, out))
        }
        Rvalue::Repeat { value, .. } => moved_by_operand(value, out),
        Rvalue::Discriminant(_) | Rvalue::Ref { .. } => {}
    }
}

fn moved_by_operand(operand: &Operand, out: &mut Vec<LocalId>) {
    if let Operand::Move(place) = operand {
        if place.projection.is_empty() {
            out.push(place.local);
        }
    }
}

/// Which locals an rvalue reads, whether by copy or by move.
fn read_by_rvalue(rvalue: &Rvalue, out: &mut Vec<LocalId>) {
    let mut push = |place: &Place| out.push(place.local);
    match rvalue {
        Rvalue::Use(o) | Rvalue::UnaryOp { operand: o, .. } => read_by_operand(o, &mut push),
        Rvalue::Cast { operand, .. } => read_by_operand(operand, &mut push),
        Rvalue::BinaryOp { lhs, rhs, .. } => {
            read_by_operand(lhs, &mut push);
            read_by_operand(rhs, &mut push);
        }
        Rvalue::Aggregate { operands, .. } => {
            operands.iter().for_each(|o| read_by_operand(o, &mut push))
        }
        Rvalue::Repeat { value, .. } => read_by_operand(value, &mut push),
        // Reading a tag or taking an address does not consume.
        Rvalue::Discriminant(_) | Rvalue::Ref { .. } => {}
    }
}

fn read_by_operand(operand: &Operand, push: &mut impl FnMut(&Place)) {
    match operand {
        Operand::Copy(place) | Operand::Move(place) => push(place),
        Operand::Const(_) => {}
    }
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

struct Reporter<'a> {
    sink: &'a mut Sink,
    errors: usize,
    /// One diagnostic per local, so a moved value read in a loop is reported
    /// once rather than every iteration.
    reported: Vec<bool>,
}

impl Reporter<'_> {
    fn report(&mut self, local: LocalId, state: Owned, span: Span, body: &Body) {
        let index = local.0 as usize;
        if self.reported[index] {
            return;
        }
        // A compiler temporary that is `Moved` is one this pass has not been
        // taught about; only user variables are reported.
        let decl = body.local(local);
        if decl.kind == LocalKind::Temp || decl.name.is_none() {
            return;
        }
        self.reported[index] = true;
        self.errors += 1;
        let name = decl.name.clone().unwrap_or_else(|| format!("_{}", local.0));
        let message = match state {
            Owned::Moved => format!("`{name}` has been moved out of"),
            _ => format!("`{name}` may have been moved out of"),
        };
        self.sink.emit_classified(
            Diagnostic::error(codes::E3040, span, message)
                .primary_label("used here after the move")
                .note("a move gives the value away; the old owner cannot use it again")
                .help("clone the value, or borrow it instead of moving it"),
        );
    }

    /// `[BRW-7]` — "no borrow of a moved or uninitialised place. `E3050`."
    /// Shape O6, whose required help is to name the path the place is not
    /// initialised on and to move the borrow after the initialisation.
    fn report_borrow(&mut self, local: LocalId, state: Owned, span: Span, body: &Body) {
        let index = local.0 as usize;
        if self.reported[index] {
            return;
        }
        let decl = body.local(local);
        if decl.kind == LocalKind::Temp || decl.name.is_none() {
            return;
        }
        self.reported[index] = true;
        self.errors += 1;
        let name = decl.name.clone().unwrap_or_else(|| format!("_{}", local.0));
        let (message, note) = match state {
            Owned::Moved => (
                format!("`{name}` is borrowed after it has been moved out of"),
                "the borrow would point at memory whose owner gave it away",
            ),
            _ => (
                format!("`{name}` may have been moved out of when it is borrowed here"),
                "on at least one path reaching this line the value is gone",
            ),
        };
        self.sink.emit_classified(
            Diagnostic::error(codes::E3050, span, message)
                .primary_label("borrowed here")
                .secondary(decl.span, format!("`{name}` is declared here"))
                .note(note)
                .help(
                    "borrow before the move, or clone the value so the move takes the copy",
                ),
        );
    }
}

/// Run over every body.
pub fn elaborate_all(bodies: &mut [Body], types: &TypeTable, sink: &mut Sink) -> usize {
    bodies.iter_mut().map(|body| elaborate(body, types, sink)).sum()
}
