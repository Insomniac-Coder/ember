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
    Body, LocalDecl, LocalId, LocalKind, Operand, Place, Projection, Rvalue, Stmt, StmtKind,
    Terminator,
};
use ember_span::Span;
use ember_types::{Ty, TypeTable, TyKind};

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
    // `[DRP-5]` — a `drop` body may not move fields out of `mut self`.
    let mut errors = check_drop_moves(body, types, sink);
    // `[EXP-6]`/`[FN-1]` — moves out of borrowed places outside `drop` (D-041).
    errors += check_borrowed_moves(body, types, sink);
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
    let cyclic = blocks_in_a_cycle(body);
    let mut reporter = Reporter {
        sink,
        errors: 0,
        reported: vec![false; body.locals.len()],
        in_loop: false,
    };
    let mut plan: Vec<(usize, usize, Owned)> = Vec::new();
    for (index, entry) in block_entry.iter().enumerate() {
        if let Some(state) = entry.clone() {
            let mut state = state;
            reporter.in_loop = cyclic[index];
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
    errors += reporter.errors;

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

/// `[DRP-5]` — the `self` of a `drop` body, when this body is one.
///
/// A `drop` method's `mut self` is a `ref mut Owner` where `Owner` declares
/// `drop` (or the `Owner` itself for an owned-self spelling). Any other body,
/// including a free function named `drop`, is not a destructor.
fn drop_self(body: &Body, types: &TypeTable) -> Option<LocalId> {
    if body.name != "drop" {
        return None;
    }
    if body.arg_count != 1 {
        return None;
    }
    let decl = body.locals.get(1)?;
    if decl.kind != LocalKind::Arg {
        return None;
    }
    // A free function can also be named `drop` with one droppable parameter;
    // only a method receiver named `self` is a destructor (same test as
    // `receiver_is_a_view` in borrows.rs).
    if decl.name.as_deref() != Some("self") {
        return None;
    }
    let inner = match types.kind(decl.ty) {
        TyKind::Ref { inner, .. } => *inner,
        _ => decl.ty,
    };
    let has_drop = match types.kind(inner) {
        TyKind::Struct(id) => types.struct_def(*id).has_drop,
        TyKind::Enum(id) => types.enum_def(*id).has_drop,
        _ => false,
    };
    has_drop.then_some(LocalId(1))
}

/// Whether a move leaves through `drop`'s `self`: rooted at it, past the borrow.
fn is_self_interior(place: &Place, self_local: LocalId) -> bool {
    place.local == self_local && !place.projection.is_empty()
}

/// `[DRP-5]`, `[EXP-6]` — a `drop` body may not move out through `mut self`.
///
/// A field move (`E3010`) double-destroys: the moved value drops with its new
/// owner at the end of the `drop` body, then drops again as a field after
/// `drop` returns. A whole-value move through the borrow (`E3013`) is the same
/// shape through `[EXP-6]` and would also launder `self` into an owned local
/// for a second field move. Only `Move` matters: `Copy` fields stay `Copy`.
fn check_drop_moves(body: &Body, types: &TypeTable, sink: &mut Sink) -> usize {
    let Some(self_local) = drop_self(body, types) else {
        return 0;
    };
    let mut errors = 0;
    let mut report = |place: &Place, span: Span| {
        let field_move =
            place.projection.iter().any(|p| !matches!(p, Projection::Deref));
        if field_move {
            sink.emit_classified(
                Diagnostic::error(
                    codes::E3010,
                    span,
                    "cannot move a field out of `drop`'s `mut self`",
                )
                .primary_label("moved out here")
                .note("a `drop` method's `mut self` may not move fields out [DRP-5]; the field is dropped again after `drop` returns [EXP-6]")
                .help("use `mem.take`, `mem.replace` or `swap` to leave a value in its place"),
            );
        } else {
            sink.emit_classified(
                Diagnostic::error(codes::E3013, span, "cannot move out of `drop`'s `mut self`")
                    .primary_label("moved out here")
                    .note("a `drop` method's `mut self` may not be moved out of [DRP-5]; the value is dropped again after `drop` returns [EXP-6]")
                    .help("borrow it instead of moving it; use `mem.take`, `mem.replace` or `swap` to leave a value in a field's place"),
            );
        }
        errors += 1;
    };
    let check_operand = |operand: &Operand, span: Span, report: &mut dyn FnMut(&Place, Span)| {
        if let Operand::Move(place) = operand {
            if is_self_interior(place, self_local) {
                report(place, span);
            }
        }
    };
    for block in &body.blocks {
        for stmt in &block.stmts {
            match &stmt.kind {
                StmtKind::Assign { rvalue, .. } => match rvalue {
                    Rvalue::Use(o) | Rvalue::UnaryOp { operand: o, .. } => {
                        check_operand(o, stmt.span, &mut report);
                    }
                    Rvalue::Cast { operand, .. } => {
                        check_operand(operand, stmt.span, &mut report);
                    }
                    Rvalue::BinaryOp { lhs, rhs, .. } => {
                        check_operand(lhs, stmt.span, &mut report);
                        check_operand(rhs, stmt.span, &mut report);
                    }
                    Rvalue::Aggregate { operands, .. } => {
                        for operand in operands {
                            check_operand(operand, stmt.span, &mut report);
                        }
                    }
                    Rvalue::Repeat { value, .. } => {
                        check_operand(value, stmt.span, &mut report);
                    }
                    Rvalue::Discriminant(_) | Rvalue::Ref { .. } => {}
                },
                StmtKind::CheckedBinaryOp { lhs, rhs, .. } => {
                    check_operand(lhs, stmt.span, &mut report);
                    check_operand(rhs, stmt.span, &mut report);
                }
                _ => {}
            }
        }
        let span = block.terminator_span;
        match &block.terminator {
            Terminator::Call { func, args, .. } => {
                if let ember_mir::FuncRef::Indirect(operand) = func {
                    check_operand(operand, span, &mut report);
                }
                for arg in args {
                    check_operand(arg, span, &mut report);
                }
            }
            Terminator::SwitchInt { discr, .. } => {
                check_operand(discr, span, &mut report);
            }
            Terminator::Assert { cond, .. } => {
                check_operand(cond, span, &mut report);
            }
            Terminator::Goto(_) | Terminator::Return | Terminator::Unreachable => {}
        }
    }
    errors
}

/// The type a place denotes, following its projections (cf. `place_ty` in
/// `verify.rs`). A projection that does not apply leaves the type alone: the
/// type checker has already rejected such a program.
fn moved_place_ty(place: &Place, body: &Body, types: &TypeTable) -> Ty {
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
            (Projection::Field(i), TyKind::Tuple(items)) => {
                ty = items.get(*i).copied().unwrap_or(ty);
            }
            (
                Projection::Index(_) | Projection::ConstIndex(_) | Projection::Column(_),
                TyKind::Array { elem, .. } | TyKind::Vec { elem } | TyKind::Span { elem, .. },
            ) => ty = *elem,
            (Projection::Deref, TyKind::Ref { inner, .. } | TyKind::Ptr { inner, .. }) => {
                ty = *inner
            }
            _ => {}
        }
    }
    ty
}

/// Whether a `Move` place goes through a safe reference (`ref`/`ref mut`).
///
/// `[EXP-6]` names `E3013` for moves out of a `ref`/`ref mut`. A deref of a
/// raw pointer is `unsafe` territory and is left to `[UNS-*]`: this answers
/// which one the first `Deref` in the chain goes through, walking the type
/// alongside the projections the way `place_ty` in `verify.rs` does.
fn deref_through_ref(place: &Place, body: &Body, types: &TypeTable) -> bool {
    let mut ty = body.local(place.local).ty;
    let mut variant = None;
    for projection in &place.projection {
        match (projection, types.kind(ty)) {
            (Projection::Deref, TyKind::Ref { .. }) => return true,
            (Projection::Deref, TyKind::Ptr { .. }) => return false,
            // Unreachable in verified MIR: only references and raw pointers
            // deref. Fail closed — a deref in safe code is a borrow.
            (Projection::Deref, _) => return true,
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
            (Projection::Field(i), TyKind::Tuple(items)) => {
                ty = items.get(*i).copied().unwrap_or(ty);
            }
            (
                Projection::Index(_) | Projection::ConstIndex(_) | Projection::Column(_),
                TyKind::Array { elem, .. } | TyKind::Vec { elem } | TyKind::Span { elem, .. },
            ) => ty = *elem,
            _ => {}
        }
    }
    false
}

/// `[EXP-6]`, `[FN-1]` — moves out of borrowed places, outside `drop` bodies.
///
/// Two shapes, one code (`E3013`, shape O2):
///
/// * a `Move` through a `Deref` — `x = r.inner` where `r: ref Outer`. The
///   referent still owns the value and drops it, so the new owner drops it a
///   second time. `[BRW-1]` forbids moving while shared borrows are live;
///   `[EXP-6]` forbids moving out of a reference unconditionally, which is
///   what is checked here rather than loan liveness.
/// * a `Move` out of a borrowed (`[FN-1]`) parameter — whole (`x = o`) or
///   field (`x = o.inner`). "The callee reads through a `ref A` … The
///   callee cannot mutate or move `a`." A borrowed parameter arrives as a
///   bitwise copy with no loan behind it, so no other analysis can see that
///   the caller still owns (and drops) the value; the move double-destroys
///   across the call boundary.
///
/// Only owning `Move`s matter: a `Copy` leaves the source intact, which is
/// why reads and copies through a shared borrow stay legal — and a move of a
/// value that owns nothing (`needs_drop` false) is a bitwise copy with no
/// second destruction to happen, so it stays legal too. Only `Ref` derefs: a
/// raw pointer is `unsafe` territory. A move of a `ref`/`ref mut` local itself
/// (no projection) transfers the borrow and stays legal, as does a move of a
/// borrowed view-typed parameter (`ref`, `Span`, `str` are `Copy`;
/// `ref mut`/`MutSpan[T]` reborrows travel as whole-local moves). `drop`'s
/// own `self` keeps `check_drop_moves`' specific messages and is skipped
/// here, so one move is never two errors.
fn check_borrowed_moves(body: &Body, types: &TypeTable, sink: &mut Sink) -> usize {
    let drop_self_local = drop_self(body, types);
    let mut errors = 0;
    let mut report = |place: &Place, span: Span| {
        if drop_self_local == Some(place.local) {
            return;
        }
        // Only ownership moves are rejected: a move of a value that owns
        // nothing is a bitwise copy the analyses already treat as one (D-041
        // is the double-destroy, and a `Panel` of `i32`s has nothing to
        // destroy twice — `retitle` in `tests/conformance/MOD-2/` moves a
        // borrowed one on every run). `Copy` types never reach here at all:
        // lowering emits `Copy` for them.
        if !types.needs_drop(moved_place_ty(place, body, types)) {
            return;
        }
        let through_ref = place
            .projection
            .iter()
            .any(|p| matches!(p, Projection::Deref))
            && deref_through_ref(place, body, types);
        let borrowed = body.borrowed_params.contains(&place.local)
            && !types.is_view(body.local(place.local).ty);
        if through_ref {
            sink.emit_classified(
                Diagnostic::error(codes::E3013, span, "cannot move out of a reference")
                    .primary_label("moved out here")
                    .note("moving out of a `ref`/`ref mut` is forbidden [EXP-6]; the referent still owns the value and drops it")
                    .help("borrow it instead of moving it; if the function needs ownership, take an `owned` parameter"),
            );
            errors += 1;
        } else if borrowed {
            sink.emit_classified(
                Diagnostic::error(codes::E3013, span, "cannot move out of a borrowed parameter")
                    .primary_label("moved out here")
                    .note("a borrowed parameter is owned by its caller [FN-1]; moving it out destroys the value twice — once here, once with the caller")
                    .help("take an `owned` parameter if the function needs ownership, or borrow the value instead of moving it"),
            );
            errors += 1;
        }
    };
    let check_operand = |operand: &Operand, span: Span, report: &mut dyn FnMut(&Place, Span)| {
        if let Operand::Move(place) = operand {
            report(place, span);
        }
    };
    for block in &body.blocks {
        for stmt in &block.stmts {
            match &stmt.kind {
                StmtKind::Assign { rvalue, .. } => match rvalue {
                    Rvalue::Use(o) | Rvalue::UnaryOp { operand: o, .. } => {
                        check_operand(o, stmt.span, &mut report);
                    }
                    Rvalue::Cast { operand, .. } => {
                        check_operand(operand, stmt.span, &mut report);
                    }
                    Rvalue::BinaryOp { lhs, rhs, .. } => {
                        check_operand(lhs, stmt.span, &mut report);
                        check_operand(rhs, stmt.span, &mut report);
                    }
                    Rvalue::Aggregate { operands, .. } => {
                        for operand in operands {
                            check_operand(operand, stmt.span, &mut report);
                        }
                    }
                    Rvalue::Repeat { value, .. } => {
                        check_operand(value, stmt.span, &mut report);
                    }
                    Rvalue::Discriminant(_) | Rvalue::Ref { .. } => {}
                },
                StmtKind::CheckedBinaryOp { lhs, rhs, .. } => {
                    check_operand(lhs, stmt.span, &mut report);
                    check_operand(rhs, stmt.span, &mut report);
                }
                StmtKind::StorageLive(_)
                | StmtKind::StorageDead(_)
                | StmtKind::Drop { .. }
                | StmtKind::Nop => {}
            }
        }
        let span = block.terminator_span;
        match &block.terminator {
            Terminator::Call { func, args, .. } => {
                if let ember_mir::FuncRef::Indirect(operand) = func {
                    check_operand(operand, span, &mut report);
                }
                for arg in args {
                    check_operand(arg, span, &mut report);
                }
            }
            Terminator::SwitchInt { discr, .. } => {
                check_operand(discr, span, &mut report);
            }
            Terminator::Assert { cond, .. } => {
                check_operand(cond, span, &mut report);
            }
            Terminator::Goto(_) | Terminator::Return | Terminator::Unreachable => {}
        }
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

/// Which blocks can reach themselves.
///
/// `[OWN-4]` is about a loop, and a loop in MIR is a block reachable from its
/// own successors — there is no `while` left by this point, only the graph.
/// Computed once per body: the bodies here are small enough that a reachability
/// walk per block is cheaper than the machinery to avoid one.
fn blocks_in_a_cycle(body: &Body) -> Vec<bool> {
    let count = body.blocks.len();
    let mut cyclic = vec![false; count];
    for start in 0..count {
        let mut seen = vec![false; count];
        let mut stack = successors(body, start);
        while let Some(next) = stack.pop() {
            if next == start {
                cyclic[start] = true;
                break;
            }
            if seen[next] {
                continue;
            }
            seen[next] = true;
            stack.extend(successors(body, next));
        }
    }
    cyclic
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
    /// `[OWN-4]` — whether the block being reported on can reach itself.
    ///
    /// This is what tells a *move in a loop* from a *use after move*, and they
    /// are different rules with different codes and different advice. Set
    /// before each block rather than threaded through `step`, which does not
    /// otherwise need to know where it is.
    in_loop: bool,
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
        // `[OWN-4]` — "A loop body that moves a value declared outside the loop
        // is `E3041` unless the value is reassigned before the next iteration
        // on every path." That is a different rule from `[OWN-3]`'s use after
        // move, and `[DIA-7a]` keys it to a different shape: O3, "move in a
        // loop", whose help is about the next iteration rather than about this
        // use.
        //
        // The two are told apart by where the use is and what the analysis
        // knows. A move and a use in one iteration leaves the local definitely
        // `Moved`, which is O1. A move that reaches its own use round a back
        // edge leaves it `MaybeMoved` at the loop head, because the entry state
        // joins "not yet moved" with "moved last time" — so `MaybeMoved` inside
        // a cycle is exactly `[OWN-4]`'s shape.
        let in_loop = self.in_loop && state != Owned::Moved;
        let diagnostic = if in_loop {
            Diagnostic::error(
                codes::E3041,
                span,
                format!("`{name}` is moved in a loop"),
            )
            .primary_label("moved here, and the loop comes back")
            .note(concat!(
                "the first iteration moves it and the second finds it gone; a value ",
                "moved in a loop must be put back before the next iteration [OWN-4]"
            ))
            .help(concat!(
                "reassign it before the end of the loop body on every path, or borrow ",
                "it instead of moving it, or move a clone"
            ))
        } else {
            let message = match state {
                Owned::Moved => format!("`{name}` has been moved out of"),
                _ => format!("`{name}` may have been moved out of"),
            };
            Diagnostic::error(codes::E3040, span, message)
                .primary_label("used here after the move")
                .note("a move gives the value away; the old owner cannot use it again")
                .help("clone the value, or borrow it instead of moving it")
        };
        self.sink.emit_classified(diagnostic);
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
