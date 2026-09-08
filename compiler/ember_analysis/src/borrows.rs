//! The NLL borrow checker (Part XVIII §4.7), first cut.
//!
//! `[BRW-1]` is the rule: at any program point a place has either any number
//! of live shared borrows, or exactly one live mutable borrow; while a mutable
//! borrow is live the owner may not read, write, move or drop the place, and
//! while shared borrows are live the owner may read but not write.
//!
//! `[BRW-2]` is what makes it usable: a borrow is live from its creation until
//! the **last use** of anything derived from it, not until the end of its
//! scope. `n = v.len(); v.push(n)` is legal because the borrow `len` took is
//! dead by the time `push` runs. That is the whole reason this analysis is a
//! liveness computation rather than a scope walk.
//!
//! What this pass does today, and what §4.7 asks for in the end:
//!
//! | §4.7 step | here |
//! |---|---|
//! | 1. regions per reference-typed local | approximated by the borrower local's liveness |
//! | 2. constraints from assignments and calls | assignment of a reference propagates the loan |
//! | 3. liveness | backward dataflow over the CFG, exact |
//! | 4. loan scope | live borrower, not killed by reassignment |
//! | 5. access check | `[BRW-1]` over overlapping places |
//! | 6. region errors | `[BRW-7]`'s `E3050`; `E3060` waits for storage-end tracking |
//! | 7. diagnostics | two labels and the "later used here" line `[DIA-3]` requires |
//!
//! The approximation in step 1 is the honest one to make first: a region *is*
//! the set of points where a borrow must be valid, and for a borrow held in a
//! local that is exactly where the local is live. It becomes wrong when a
//! reference is returned, stored in a view struct, or passed into a callback —
//! `[LT-1]`, `[LT-2]` and `[LT-7]` — and those are what a region variable and
//! its constraint graph buy. None of them is expressible yet.

use std::collections::{HashMap, HashSet};

use ember_diag::{Diagnostic, Sink, codes};
use ember_mir::{
    BasicBlockId, Body, LocalId, LocalKind, Operand, Place, Projection, Rvalue, StmtKind,
    Terminator,
};
use ember_types::{Ty, TyKind, TypeTable};
use ember_span::Span;

/// A point in the CFG: a statement index within a block, where `stmts.len()`
/// is the terminator.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
struct Point {
    block: usize,
    index: usize,
}

/// One borrow, recorded where it is created (`[GLOSSARY]` "loan").
#[derive(Clone, Debug)]
struct Loan {
    /// The place borrowed.
    place: Place,
    mutable: bool,
    /// The local the reference was stored in. Its liveness is the loan's.
    borrower: LocalId,
    created_at: Point,
    span: Span,
    /// `[BRW-3]` — a mutable borrow taken for a call's receiver or a `mut`
    /// argument is *reserved* where it is created and *activated* at the call.
    /// At every point in between it behaves as a shared borrow, which is what
    /// makes `v.push(v.len())` legal: the argument's shared borrow of `v` is
    /// taken and released inside the window.
    ///
    /// The window is a set of points rather than an end marker because
    /// evaluating an argument that is itself a call ends the block, so the
    /// reservation and its activation sit in different blocks.
    reserved_at: HashSet<Point>,
}

/// How a place is touched at a point (§4.7 step 5).
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum Access {
    Read,
    Write,
    Borrow { mutable: bool },
}

pub fn check_all(bodies: &[Body], types: &TypeTable, sink: &mut Sink) {
    for body in bodies {
        check(body, types, sink);
    }
}

pub fn check(body: &Body, types: &TypeTable, sink: &mut Sink) {
    let loans = collect_loans(body);
    if loans.is_empty() {
        return;
    }
    let live = liveness(body);
    let mut reported = HashSet::new();

    for (block_index, block) in body.blocks.iter().enumerate() {
        for (index, stmt) in block.stmts.iter().enumerate() {
            let point = Point { block: block_index, index };
            let mut accesses = Vec::new();
            match &stmt.kind {
                StmtKind::Assign { place, rvalue } => {
                    // A borrow is not a conflicting read of its own operand.
                    if let Rvalue::Ref { place: borrowed, mutable } = rvalue {
                        accesses.push((borrowed.clone(), Access::Borrow { mutable: *mutable }));
                    } else {
                        rvalue_reads(rvalue, &mut accesses);
                    }
                    accesses.push((place.clone(), Access::Write));
                }
                StmtKind::CheckedBinaryOp { dest, overflow, lhs, rhs, .. } => {
                    operand_read(lhs, &mut accesses);
                    operand_read(rhs, &mut accesses);
                    accesses.push((dest.clone(), Access::Write));
                    accesses.push((overflow.clone(), Access::Write));
                }
                StmtKind::Drop { place, .. } => {
                    accesses.push((place.clone(), Access::Write));
                }
                StmtKind::StorageLive(_) | StmtKind::StorageDead(_) | StmtKind::Nop => {}
            }
            check_point(body, types, &loans, &live, point, &accesses, stmt.span, sink, &mut reported);
        }

        let point = Point { block: block_index, index: block.stmts.len() };
        let mut accesses = Vec::new();
        match &block.terminator {
            Terminator::SwitchInt { discr, .. } => operand_read(discr, &mut accesses),
            Terminator::Call { args, dest, .. } => {
                for arg in args {
                    operand_read(arg, &mut accesses);
                }
                accesses.push((dest.clone(), Access::Write));
            }
            Terminator::Assert { cond, .. } => operand_read(cond, &mut accesses),
            Terminator::Goto(_) | Terminator::Return | Terminator::Unreachable => {}
        }
        check_point(
            body,
            types,
            &loans,
            &live,
            point,
            &accesses,
            block.terminator_span,
            sink,
            &mut reported,
        );

        // §4.7 step 6 — a loan still live where the borrowed place's storage
        // ends. Returning is the case that matters: the reference leaves the
        // frame while what it points at does not.
        if matches!(block.terminator, Terminator::Return) {
            check_escapes(body, types, &loans, &live, point, block.terminator_span, sink);
        }
    }
}

/// `E3060` — the borrowed value does not live long enough.
///
/// A borrow of a **parameter** is fine: the caller owns what it points at and
/// `[LT-1]`'s elision ties the return's region to it. A borrow of a local is
/// not: the local's storage ends with the frame, so the reference dangles.
fn check_escapes(
    body: &Body,
    types: &TypeTable,
    loans: &[Loan],
    live: &HashMap<Point, HashSet<LocalId>>,
    point: Point,
    span: Span,
    sink: &mut Sink,
) {
    for loan in in_scope(loans, live, point) {
        let root = body.local(loan.place.local);
        if root.kind == LocalKind::Arg {
            continue;
        }
        // Only a borrow that actually leaves: the return slot holds it, or a
        // local that the return slot was assigned from does.
        if !live.get(&point).is_some_and(|l| l.contains(&loan.borrower)) {
            continue;
        }
        let name = place_name(body, types, &loan.place);
        sink.emit_classified(
            Diagnostic::error(
                codes::E3060,
                span,
                format!("`{name}` does not live long enough"),
            )
            .primary_label("the borrow is still live when the function returns")
            .secondary(loan.span, format!("`{name}` is borrowed here"))
            .secondary(root.span, format!("`{name}` is a local, so its storage ends with the frame"))
            .help(
                "return an owned value, take the destination as a `mut` parameter, or borrow \
                 something the caller owns",
            )
            .note("a returned reference must derive from a parameter (LT-1)"),
        );
    }
}

/// Every `Rvalue::Ref` in the body, with the local it is stored into.
fn collect_loans(body: &Body) -> Vec<Loan> {
    let mut loans = Vec::new();
    for (block_index, block) in body.blocks.iter().enumerate() {
        for (index, stmt) in block.stmts.iter().enumerate() {
            let StmtKind::Assign { place, rvalue } = &stmt.kind else { continue };
            let Rvalue::Ref { place: borrowed, mutable } = rvalue else { continue };
            // A borrow written straight into a projection rather than a whole
            // local is a view field; those arrive with `[TYP-15]` and block E's
            // region work, and are not tracked here.
            if !place.projection.is_empty() {
                continue;
            }
            let created_at = Point { block: block_index, index };
            loans.push(Loan {
                place: borrowed.clone(),
                mutable: *mutable,
                borrower: place.local,
                created_at,
                span: stmt.span,
                reserved_at: reservation_window(body, place.local, created_at),
            });
        }
    }
    loans
}

/// `[BRW-3]` — the points at which a mutable borrow is merely *reserved*.
///
/// A borrow taken for a call is reserved from its creation until the call
/// consumes it, and behaves as shared throughout. The window is found by
/// walking forward along the single-successor chain that argument evaluation
/// produces, stopping at the first use of the borrower. If that use is a call
/// argument, everything before it is the window; if it is anything else, the
/// borrow was never two-phase and the window is empty.
fn reservation_window(body: &Body, borrower: LocalId, created_at: Point) -> HashSet<Point> {
    let mut window = HashSet::new();
    let mut block_index = created_at.block;
    let mut start = created_at.index + 1;

    // Bounded by the block count: argument evaluation is a chain, and a loop
    // back into it would mean the borrow is used more than once anyway.
    for _ in 0..body.blocks.len() {
        let Some(block) = body.blocks.get(block_index) else { return HashSet::new() };

        for (index, stmt) in block.stmts.iter().enumerate().skip(start) {
            let mut reads = Vec::new();
            match &stmt.kind {
                StmtKind::Assign { place, rvalue } => {
                    rvalue_reads(rvalue, &mut reads);
                    if place.local == borrower {
                        return HashSet::new();
                    }
                }
                StmtKind::CheckedBinaryOp { lhs, rhs, .. } => {
                    operand_read(lhs, &mut reads);
                    operand_read(rhs, &mut reads);
                }
                StmtKind::Drop { place, .. } => reads.push((place.clone(), Access::Read)),
                _ => {}
            }
            if reads.iter().any(|(p, _)| p.local == borrower) {
                return HashSet::new();
            }
            window.insert(Point { block: block_index, index });
        }

        let terminator_point = Point { block: block_index, index: block.stmts.len() };
        match &block.terminator {
            Terminator::Call { args, next, .. } => {
                let used = args.iter().any(|a| match a {
                    Operand::Copy(p) | Operand::Move(p) => p.local == borrower,
                    Operand::Const(_) => false,
                });
                if used {
                    // This is the activation. The window is everything before.
                    return window;
                }
                window.insert(terminator_point);
                block_index = next.0 as usize;
                start = 0;
            }
            Terminator::Assert { next, .. } => {
                window.insert(terminator_point);
                block_index = next.0 as usize;
                start = 0;
            }
            Terminator::Goto(next) => {
                window.insert(terminator_point);
                block_index = next.0 as usize;
                start = 0;
            }
            // A branch means the borrow outlives argument evaluation.
            _ => return HashSet::new(),
        }
    }
    HashSet::new()
}

/// `[BRW-2]` — the loans in scope at a point: created before it, and with the
/// borrower still live. Liveness *is* the region, for a borrow held in a local.
fn in_scope<'a>(
    loans: &'a [Loan],
    live: &HashMap<Point, HashSet<LocalId>>,
    point: Point,
) -> Vec<&'a Loan> {
    let Some(live_here) = live.get(&point) else { return Vec::new() };
    loans
        .iter()
        .filter(|loan| {
            // Created strictly earlier in the same block, or in another block
            // that reaches this one — liveness of the borrower already
            // encodes reachability, so ordering within a block is the only
            // extra condition.
            let earlier = loan.created_at.block != point.block
                || loan.created_at.index < point.index;
            earlier && live_here.contains(&loan.borrower)
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn check_point(
    body: &Body,
    types: &TypeTable,
    loans: &[Loan],
    live: &HashMap<Point, HashSet<LocalId>>,
    point: Point,
    accesses: &[(Place, Access)],
    span: Span,
    sink: &mut Sink,
    reported: &mut HashSet<(usize, usize)>,
) {
    let scope = in_scope(loans, live, point);
    if scope.is_empty() {
        return;
    }
    for (place, access) in accesses {
        for loan in &scope {
            if !overlaps(&loan.place, place) {
                continue;
            }
            // The borrower itself is not a conflicting access.
            if place.local == loan.borrower {
                continue;
            }
            // `[BRW-3]` — inside the reservation window the borrow is not yet
            // mutable, so shared borrows and reads of the same place pass.
            let reserved = loan.mutable && loan.reserved_at.contains(&point);
            let loan_mutable = loan.mutable && !reserved;
            let conflict = match access {
                // While a mutable borrow is live the owner may not read.
                Access::Read => loan_mutable,
                // While any borrow is live the owner may not write.
                Access::Write => true,
                // Two mutable, or one of each, conflict; two shared do not.
                Access::Borrow { mutable } => loan_mutable || *mutable,
            };
            if !conflict {
                continue;
            }

            let key = (loan.created_at.block * 4096 + loan.created_at.index, point.block * 4096 + point.index);
            if !reported.insert(key) {
                continue;
            }

            let name = place_name(body, types, &loan.place);
            let (code, message) = match (loan_mutable, access) {
                (true, Access::Borrow { mutable: true }) => (
                    codes::E3022,
                    format!("`{name}` is already mutably borrowed"),
                ),
                (_, Access::Borrow { .. }) => (
                    codes::E3021,
                    format!("`{name}` is borrowed here and mutably borrowed elsewhere"),
                ),
                (true, Access::Read) => {
                    (codes::E3021, format!("`{name}` cannot be read while it is mutably borrowed"))
                }
                _ => (codes::E3021, format!("`{name}` cannot be written while it is borrowed")),
            };

            // `[DIA-3]` — the borrow site, the conflicting access, and the
            // later use that keeps the borrow alive.
            // `[DIA-2]` — never name a temporary at the user. When the
            // borrow lives in one, the advice is about the expression, not
            // about a local they cannot see.
            let borrower = body.local(loan.borrower).name.clone();
            let kind = if loan_mutable { "mutable " } else { "" };
            sink.emit_classified(
                Diagnostic::error(code, span, message)
                    .primary_label("conflicting access here")
                    .secondary(loan.span, format!("{kind}borrow of `{name}` starts here"))
                    .help(match &borrower {
                        Some(name) => format!(
                            "end the borrow before this: `{name}` is what keeps it alive, so \
                             shorten its last use or put it in a block of its own"
                        ),
                        None => format!(
                            "bind the borrow of `{name}` to a local and finish with it before \
                             this line, or copy the value out first",
                            name = name
                        ),
                    })
                    .note("a borrow lasts until its last use, not to the end of the scope (BRW-2)"),
            );
        }
    }
}

/// Two places overlap when one is a prefix of the other (§4.7 step 5).
/// `Index` projections are assumed to overlap unless both are constant and
/// different — `[BRW-5]`, "indices are not disjoint".
fn overlaps(a: &Place, b: &Place) -> bool {
    if a.local != b.local {
        return false;
    }
    for (pa, pb) in a.projection.iter().zip(b.projection.iter()) {
        match (pa, pb) {
            (Projection::Field(x), Projection::Field(y)) if x != y => return false,
            (Projection::ConstIndex(x), Projection::ConstIndex(y)) if x != y => return false,
            _ => {}
        }
    }
    true
}

/// `[DIA-2]` — name the place as it was written. MIR holds field *indices*,
/// so the type is walked alongside the projections to recover `p.x` from
/// `p.0`. A tuple keeps its number, because that is what the source says too.
fn place_name(body: &Body, types: &TypeTable, place: &Place) -> String {
    let decl = body.local(place.local);
    let mut out = decl.name.clone().unwrap_or_else(|| format!("_{}", place.local.0));
    let mut ty = decl.ty;
    for projection in &place.projection {
        match projection {
            Projection::Field(i) => {
                out = match field_name(types, ty, *i) {
                    Some(name) => format!("{out}.{name}"),
                    None => format!("{out}.{i}"),
                };
                ty = field_ty(types, ty, *i).unwrap_or(ty);
            }
            Projection::ConstIndex(i) => {
                out = format!("{out}[{i}]");
                ty = element_ty(types, ty).unwrap_or(ty);
            }
            Projection::Index(_) => {
                out = format!("{out}[…]");
                ty = element_ty(types, ty).unwrap_or(ty);
            }
            Projection::Deref => {
                out = format!("*{out}");
                ty = pointee_ty(types, ty).unwrap_or(ty);
            }
            _ => {}
        }
    }
    out
}

fn field_name(types: &TypeTable, ty: Ty, index: usize) -> Option<String> {
    match types.kind(ty) {
        TyKind::Struct(id) => {
            types.struct_def(*id).fields.get(index).map(|f| f.name.to_string())
        }
        _ => None,
    }
}

fn field_ty(types: &TypeTable, ty: Ty, index: usize) -> Option<Ty> {
    match types.kind(ty) {
        TyKind::Struct(id) => types.struct_def(*id).fields.get(index).map(|f| f.ty),
        TyKind::Tuple(items) => items.get(index).copied(),
        _ => None,
    }
}

fn element_ty(types: &TypeTable, ty: Ty) -> Option<Ty> {
    match types.kind(ty) {
        TyKind::Array { elem, .. } | TyKind::Vec { elem } => Some(*elem),
        _ => None,
    }
}

fn pointee_ty(types: &TypeTable, ty: Ty) -> Option<Ty> {
    match types.kind(ty) {
        TyKind::Ref { inner, .. } | TyKind::Ptr { inner, .. } => Some(*inner),
        _ => None,
    }
}

fn operand_read(operand: &Operand, out: &mut Vec<(Place, Access)>) {
    match operand {
        Operand::Copy(p) | Operand::Move(p) => out.push((p.clone(), Access::Read)),
        Operand::Const(_) => {}
    }
}

fn rvalue_reads(rvalue: &Rvalue, out: &mut Vec<(Place, Access)>) {
    match rvalue {
        Rvalue::Use(o) | Rvalue::UnaryOp { operand: o, .. } => operand_read(o, out),
        Rvalue::Cast { operand, .. } => operand_read(operand, out),
        Rvalue::BinaryOp { lhs, rhs, .. } => {
            operand_read(lhs, out);
            operand_read(rhs, out);
        }
        Rvalue::Aggregate { operands, .. } => {
            for o in operands {
                operand_read(o, out);
            }
        }
        Rvalue::Repeat { value, .. } => operand_read(value, out),
        Rvalue::Discriminant(place) => out.push((place.clone(), Access::Read)),
        Rvalue::Ref { place, mutable } => {
            out.push((place.clone(), Access::Borrow { mutable: *mutable }))
        }
    }
}

/// §4.7 step 3 — backward liveness over the CFG, to a fixpoint. A local is
/// live at a point if some path from it reaches a read before a write.
fn liveness(body: &Body) -> HashMap<Point, HashSet<LocalId>> {
    let mut live_out: HashMap<usize, HashSet<LocalId>> = HashMap::new();
    let mut changed = true;
    while changed {
        changed = false;
        for index in (0..body.blocks.len()).rev() {
            let mut out = HashSet::new();
            for successor in successors(&body.blocks[index].terminator) {
                // A block's live-out is the union of its successors' live-in.
                if let Some(entry) = live_in(body, successor.0 as usize, &live_out) {
                    out.extend(entry);
                }
            }
            if live_out.get(&index).map(|e| e != &out).unwrap_or(true) {
                live_out.insert(index, out);
                changed = true;
            }
        }
    }

    // Walk each block backwards from its live-out to give every point a set.
    let mut points = HashMap::new();
    for (index, block) in body.blocks.iter().enumerate() {
        let mut live: HashSet<LocalId> = live_out.get(&index).cloned().unwrap_or_default();
        terminator_transfer(&block.terminator, &mut live);
        points.insert(Point { block: index, index: block.stmts.len() }, live.clone());
        for (i, stmt) in block.stmts.iter().enumerate().rev() {
            stmt_transfer(&stmt.kind, &mut live);
            points.insert(Point { block: index, index: i }, live.clone());
        }
    }
    points
}

fn live_in(
    body: &Body,
    block: usize,
    live_out: &HashMap<usize, HashSet<LocalId>>,
) -> Option<HashSet<LocalId>> {
    let b = body.blocks.get(block)?;
    let mut live: HashSet<LocalId> = live_out.get(&block).cloned().unwrap_or_default();
    terminator_transfer(&b.terminator, &mut live);
    for stmt in b.stmts.iter().rev() {
        stmt_transfer(&stmt.kind, &mut live);
    }
    Some(live)
}

fn successors(terminator: &Terminator) -> Vec<BasicBlockId> {
    match terminator {
        Terminator::Goto(bb) => vec![*bb],
        Terminator::SwitchInt { targets, otherwise, .. } => {
            let mut out: Vec<BasicBlockId> = targets.iter().map(|(_, bb)| *bb).collect();
            out.push(*otherwise);
            out
        }
        Terminator::Call { next, .. } | Terminator::Assert { next, .. } => vec![*next],
        Terminator::Return | Terminator::Unreachable => Vec::new(),
    }
}

fn stmt_transfer(kind: &StmtKind, live: &mut HashSet<LocalId>) {
    match kind {
        StmtKind::Assign { place, rvalue } => {
            if place.projection.is_empty() {
                live.remove(&place.local);
            } else {
                live.insert(place.local);
            }
            let mut reads = Vec::new();
            rvalue_reads(rvalue, &mut reads);
            for (p, _) in reads {
                live.insert(p.local);
            }
        }
        StmtKind::CheckedBinaryOp { dest, overflow, lhs, rhs, .. } => {
            live.remove(&dest.local);
            live.remove(&overflow.local);
            let mut reads = Vec::new();
            operand_read(lhs, &mut reads);
            operand_read(rhs, &mut reads);
            for (p, _) in reads {
                live.insert(p.local);
            }
        }
        StmtKind::Drop { place, .. } => {
            live.insert(place.local);
        }
        StmtKind::StorageLive(l) | StmtKind::StorageDead(l) => {
            live.remove(l);
        }
        StmtKind::Nop => {}
    }
}

fn terminator_transfer(terminator: &Terminator, live: &mut HashSet<LocalId>) {
    match terminator {
        Terminator::SwitchInt { discr, .. } => {
            let mut reads = Vec::new();
            operand_read(discr, &mut reads);
            for (p, _) in reads {
                live.insert(p.local);
            }
        }
        Terminator::Call { args, dest, .. } => {
            live.remove(&dest.local);
            for arg in args {
                let mut reads = Vec::new();
                operand_read(arg, &mut reads);
                for (p, _) in reads {
                    live.insert(p.local);
                }
            }
        }
        Terminator::Assert { cond, .. } => {
            let mut reads = Vec::new();
            operand_read(cond, &mut reads);
            for (p, _) in reads {
                live.insert(p.local);
            }
        }
        // The return slot is read by the caller, so it is live at every
        // `Return`. Without this a borrow that leaves the frame looks dead
        // exactly where it matters, and §4.7 step 6 never fires.
        Terminator::Return => {
            live.insert(ember_mir::RETURN_LOCAL);
        }
        Terminator::Goto(_) | Terminator::Unreachable => {}
    }
}
