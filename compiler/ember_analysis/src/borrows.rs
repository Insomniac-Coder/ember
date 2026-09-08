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
    BasicBlockId, Body, LocalId, Operand, Place, Projection, Rvalue, StmtKind, Terminator,
};
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
}

/// How a place is touched at a point (§4.7 step 5).
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum Access {
    Read,
    Write,
    Borrow { mutable: bool },
}

pub fn check_all(bodies: &[Body], sink: &mut Sink) {
    for body in bodies {
        check(body, sink);
    }
}

pub fn check(body: &Body, sink: &mut Sink) {
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
            check_point(body, &loans, &live, point, &accesses, stmt.span, sink, &mut reported);
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
            &loans,
            &live,
            point,
            &accesses,
            block.terminator_span,
            sink,
            &mut reported,
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
            loans.push(Loan {
                place: borrowed.clone(),
                mutable: *mutable,
                borrower: place.local,
                created_at: Point { block: block_index, index },
                span: stmt.span,
            });
        }
    }
    loans
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
            let conflict = match access {
                // While a mutable borrow is live the owner may not read.
                Access::Read => loan.mutable,
                // While any borrow is live the owner may not write.
                Access::Write => true,
                // Two mutable, or one of each, conflict; two shared do not.
                Access::Borrow { mutable } => loan.mutable || *mutable,
            };
            if !conflict {
                continue;
            }

            let key = (loan.created_at.block * 4096 + loan.created_at.index, point.block * 4096 + point.index);
            if !reported.insert(key) {
                continue;
            }

            let name = place_name(body, &loan.place);
            let (code, message) = match (loan.mutable, access) {
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
            let borrower = place_name(body, &Place::local(loan.borrower));
            let kind = if loan.mutable { "mutable " } else { "" };
            sink.emit(
                Diagnostic::error(code, span, message)
                    .primary_label("conflicting access here")
                    .secondary(loan.span, format!("{kind}borrow of `{name}` starts here"))
                    .help(format!(
                        "end the borrow before this: `{borrower}` is what keeps it alive, so \
                         shorten its last use or put it in a block of its own"
                    ))
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

fn place_name(body: &Body, place: &Place) -> String {
    let base = body
        .local(place.local)
        .name
        .clone()
        .unwrap_or_else(|| format!("_{}", place.local.0));
    let mut out = base;
    for projection in &place.projection {
        match projection {
            Projection::Field(i) => out = format!("{out}.{i}"),
            Projection::ConstIndex(i) => out = format!("{out}[{i}]"),
            Projection::Index(_) => out = format!("{out}[…]"),
            Projection::Deref => out = format!("*{out}"),
            _ => {}
        }
    }
    out
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
        Terminator::Goto(_) | Terminator::Return | Terminator::Unreachable => {}
    }
}
