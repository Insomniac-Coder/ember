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
//! | 1. regions per reference-typed local | real region variables — `regions.rs` |
//! | 2. constraints from assignments and calls | the constraint graph, closed to a fixpoint |
//! | 3. liveness | backward dataflow over the CFG, exact |
//! | 4. loan scope | the point is in the loan's region |
//! | 5. access check | `[BRW-1]` over overlapping places |
//! | 6. region errors | `[BRW-7]`'s `E3050`; `E3060` where a loan outlives the frame |
//! | 7. diagnostics | two labels and the "later used here" line `[DIA-3]` requires |
//!
//! Step 1 was an approximation until 2026-09-09: a region *is* the set of
//! points where a borrow must be valid, and for a borrow that stays in the
//! local it was created in that is exactly where the local is live. It stopped
//! being true the moment the reference moved — a copy, a reborrow, or a call
//! handing one back — and `regions.rs` is what replaced it.

use std::collections::{HashMap, HashSet};

use ember_diag::{Diagnostic, Sink, codes};
use ember_mir::{
    BasicBlockId, Body, Builtin, FuncRef, LocalId, LocalKind, Operand, Place, Projection, Rvalue,
    StmtKind,
    Terminator,
};
use ember_types::{Ty, TyKind, TypeTable};
use ember_span::Span;

use crate::regions::{Elision, Origin, Point, RegionVid, Regions};

/// One borrow, recorded where it is created (`[GLOSSARY]` "loan").
#[derive(Clone, Debug)]
struct Loan {
    /// The place borrowed.
    place: Place,
    mutable: bool,
    /// The local the reference was stored in. Named in the help line, and
    /// exempt from being its own conflicting access.
    borrower: LocalId,
    /// §4.7 step 4 — the loan is in scope exactly where this region is.
    region: RegionVid,
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
    // `[LT-1]` is a property of the *callee's* signature, so a call in one
    // body is read against another's. The table is built once.
    let mut signatures: HashMap<&str, Elision> = HashMap::new();
    for body in bodies {
        signatures.insert(body.symbol.as_str(), elision_of(body, types));
    }
    let elision = |func: &FuncRef| match func {
        FuncRef::Direct { symbol } => {
            signatures.get(symbol.as_str()).cloned().unwrap_or(Elision::Everything)
        }
        // A builtin is `println`, `format` or an arithmetic helper: none of
        // them hands back a view of an argument.
        // `[SPN-1]` — a view built from a container points **into** it, so
        // the container stays borrowed for as long as the caller holds the
        // view. Without this the borrow checker sees no loan and
        // `v: Span[i32] = a` followed by `a.push(…)` compiles: the push
        // reallocates and `v` is dangling, which is the exact thing
        // `[UNS-4]` and `[PHIL-10]` say Safe Ember cannot do.
        FuncRef::Builtin { which: Builtin::SpanFrom { .. }, .. } => Elision::Named(vec![0]),
        // `[SPN-2]` — `get` and `get_unchecked` return a reference into the
        // view, so the view stays borrowed too.
        FuncRef::Builtin {
            which: Builtin::SpanGet | Builtin::SpanGetUnchecked,
            ..
        } => Elision::Named(vec![0]),
        FuncRef::Builtin { .. } => Elision::Nothing,
        // `[CLO-3]` — a call through a function value. `[EFF-2]`'s
        // reasoning applies to regions too: nothing is known about the
        // callee, so the permissive reading of `[LT-1]` rule 3 is taken
        // and the result is treated as borrowing every view argument.
        FuncRef::Indirect(_) => Elision::Everything,
    };
    for body in bodies {
        check_body(body, types, &elision, sink);
    }
}

/// `[LT-1]` — what a call to this function ties its result to.
///
/// Rule 1 first: a view-typed `self` receiver takes the return on its own, and
/// a method whose result points into an argument instead has to say so with
/// `@borrows` — which is `[LT-1a]`, and which overrides all three rules. Rules
/// 2 and 3 are the same answer written twice: every view-typed parameter, the
/// difference between them being which parameters exist rather than what the
/// caller must assume.
///
/// The permissiveness here is only sound because the *body* is checked against
/// the same set: `check_return_regions` rejects a return that points into a
/// parameter this said it would not.
fn elision_of(body: &Body, types: &TypeTable) -> Elision {
    if !types.is_view(body.return_ty()) {
        return Elision::Nothing;
    }
    if let Some(named) = &body.borrows {
        return Elision::Named(named.clone());
    }
    if receiver_is_a_view(body, types) {
        return Elision::Named(vec![0]);
    }
    Elision::Everything
}

/// `[LT-1]` rule 1 — a `self`/`mut self` receiver that is itself a borrow.
fn receiver_is_a_view(body: &Body, types: &TypeTable) -> bool {
    body.arg_count > 0
        && body.local(LocalId(1)).name.as_deref() == Some("self")
        && types.is_view(body.local(LocalId(1)).ty)
}

/// `[LT-1]` and `[LT-1a]`, the body half: the parameters a returned view is
/// allowed to point into, as locals.
fn allowed_origins(body: &Body, types: &TypeTable) -> Vec<LocalId> {
    if let Some(named) = &body.borrows {
        return named.iter().map(|i| LocalId(*i as u32 + 1)).collect();
    }
    if receiver_is_a_view(body, types) {
        return vec![LocalId(1)];
    }
    body.args().filter(|(_, decl)| types.is_view(decl.ty)).map(|(local, _)| local).collect()
}

/// `E3062` — the returned view points into a parameter elision did not tie it
/// to (shape B6).
///
/// This is the half of `[LT-1a]` that needed regions. The attribute was
/// checked as a *signature* — that it names parameters, that they are
/// view-typed, that the return is a view — and the body was free to contradict
/// it. `@borrows(a)` on a function that returns `b` compiled, and the caller
/// then went on using `b` while holding a reference into it.
fn check_return_regions(body: &Body, types: &TypeTable, regions: &Regions, sink: &mut Sink) {
    if !types.is_view(body.return_ty()) {
        return;
    }
    let Some(region) = regions.local_region(ember_mir::RETURN_LOCAL) else { return };
    let allowed = allowed_origins(body, types);
    let Some(span) = body
        .blocks
        .iter()
        .find(|b| matches!(b.terminator, Terminator::Return))
        .map(|b| b.terminator_span)
    else {
        return;
    };

    let mut offenders: Vec<LocalId> = regions
        .origins(region)
        .iter()
        .filter_map(|origin| match origin {
            // A borrow of a by-value parameter is not an elision question at
            // all: nothing in the caller outlives it. `check_escapes` reports
            // that as `E3060`, the same as a local.
            Origin::Param(local)
                if !allowed.contains(local) && types.is_view(body.local(*local).ty) =>
            {
                Some(*local)
            }
            _ => None,
        })
        .collect();
    offenders.sort();

    for local in offenders {
        let decl = body.local(local);
        let name = decl.name.clone().unwrap_or_else(|| format!("_{}", local.0));
        let allowed_names: Vec<String> = allowed
            .iter()
            .map(|l| match &body.local(*l).name {
                Some(name) => format!("`{name}`"),
                None => format!("`_{}`", l.0),
            })
            .collect();
        let (message, help) = if body.borrows.is_some() {
            (
                format!("the returned view points into `{name}`, which `@borrows` does not name"),
                match allowed_names.len() {
                    0 => "add `{name}` to `@borrows`".replace("{name}", &name),
                    _ => format!(
                        "add `{name}` to `@borrows`, or return a view of {}",
                        allowed_names.join(" or ")
                    ),
                },
            )
        } else {
            (
                format!("the returned view points into `{name}` rather than into `self`"),
                format!("write `@borrows({name})` above the declaration, which overrides rule 1"),
            )
        };
        sink.emit_classified(
            Diagnostic::error(codes::E3062, span, message)
                .primary_label("returned here")
                .secondary(decl.span, format!("`{name}` is the parameter it points into"))
                .help(help)
                .note("`@borrows` is what ties a return to a parameter elision would not (LT-1a)"),
        );
    }
}

pub fn check(body: &Body, types: &TypeTable, sink: &mut Sink) {
    check_body(body, types, &|_| Elision::Everything, sink);
}

fn check_body(
    body: &Body,
    types: &TypeTable,
    elision: &dyn Fn(&FuncRef) -> Elision,
    sink: &mut Sink,
) {
    let live = liveness(body);
    let regions = Regions::infer(body, types, &live, elision);
    // Before the loans: a function that hands back a parameter has no loan of
    // its own, and `[LT-1a]` is about exactly that function.
    check_return_regions(body, types, &regions, sink);
    let loans = collect_loans(body, &regions);
    if loans.is_empty() {
        return;
    }
    let reads = collect_reads(body);
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
            check_point(
                body,
                types,
                &loans,
                &regions,
                &reads,
                point,
                &accesses,
                stmt.span,
                sink,
                &mut reported,
            );
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
            &regions,
            &reads,
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
            check_escapes(body, types, &loans, &regions, point, block.terminator_span, sink);
        }
    }
}

/// `E3060` — the borrowed value does not live long enough.
///
/// A borrow of a **view-typed parameter** is fine: the caller owns what it
/// points at and `[LT-1]`'s elision ties the return's region to it. A borrow
/// of a local is not, and neither is a borrow of a parameter passed *by
/// value* — a copy lives in this frame and dies with it, whatever the caller
/// still holds. Which parameter the return may point into is `[LT-1]`'s
/// question and is `check_return_regions`'s.
fn check_escapes(
    body: &Body,
    types: &TypeTable,
    loans: &[Loan],
    regions: &Regions,
    point: Point,
    span: Span,
    sink: &mut Sink,
) {
    for loan in in_scope(loans, regions, point) {
        let root = body.local(loan.place.local);
        if root.kind == LocalKind::Arg && types.is_view(root.ty) {
            continue;
        }
        let name = place_name(body, types, &loan.place);
        // The label is about the *owner*, which for `self.n` is `self`.
        let owner = place_name(body, types, &Place::local(loan.place.local));
        let storage = match root.kind {
            LocalKind::Arg => format!(
                "`{owner}` is passed by value, so the copy's storage ends with the frame"
            ),
            _ => format!("`{owner}` is a local, so its storage ends with the frame"),
        };
        sink.emit_classified(
            Diagnostic::error(
                codes::E3060,
                span,
                format!("`{name}` does not live long enough"),
            )
            .primary_label("the borrow is still live when the function returns")
            .secondary(loan.span, format!("`{name}` is borrowed here"))
            .secondary(root.span, storage)
            .help(
                "return an owned value, take the destination as a `mut` parameter, or borrow \
                 something the caller owns",
            )
            .note("a returned reference must derive from a parameter (LT-1)"),
        );
    }
}

/// Every `Rvalue::Ref` in the body, with the region inference gave it.
fn collect_loans(body: &Body, regions: &Regions) -> Vec<Loan> {
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
            let Some(region) = regions.loan_region(created_at) else { continue };
            loans.push(Loan {
                place: borrowed.clone(),
                mutable: *mutable,
                borrower: place.local,
                region,
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
fn in_scope<'a>(loans: &'a [Loan], regions: &Regions, point: Point) -> Vec<&'a Loan> {
    loans.iter().filter(|loan| regions.contains(loan.region, point)).collect()
}

#[allow(clippy::too_many_arguments)]
fn check_point(
    body: &Body,
    types: &TypeTable,
    loans: &[Loan],
    regions: &Regions,
    reads: &HashMap<LocalId, Vec<Span>>,
    point: Point,
    accesses: &[(Place, Access)],
    span: Span,
    sink: &mut Sink,
    reported: &mut HashSet<(usize, usize)>,
) {
    let scope = in_scope(loans, regions, point);
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
            // later use that keeps the borrow alive. The last of those is not
            // necessarily the local the borrow was written into: with real
            // regions a loan is kept alive by whatever the reference reached,
            // which may be a copy of it made three lines further down.
            // `[DIA-2]` — never name a temporary at the user. When the
            // borrow lives in one, the advice is about the expression, not
            // about a local they cannot see.
            let (borrower, later, holder) = keeper(body, regions, reads, loan, span);
            // `[LT-2]` — when a view struct bundling two views is what holds
            // the loan, this is shape B13 and not B3, and `[DIA-7a]` keys it
            // to `E3064`.
            let bundled = bundles_two_views(body, types, holder);
            let (code, message) = if bundled {
                (
                    codes::E3064,
                    format!("`{name}` is borrowed through a view struct that bundles two views"),
                )
            } else {
                (code, message)
            };
            let kind = if loan_mutable { "mutable " } else { "" };
            let mut diagnostic = Diagnostic::error(code, span, message)
                .primary_label("conflicting access here")
                .secondary(loan.span, format!("{kind}borrow of `{name}` starts here"));
            if let Some(later) = later {
                diagnostic = diagnostic.secondary(later, "borrow later used here");
            }
            sink.emit_classified(
                diagnostic
                    .help(match (&borrower, bundled) {
                        // `[DIA-7a]` shape B13's help, which is a different fix
                        // from B3's: the use keeping the loan alive may be of
                        // the *other* field, so shortening it is no answer.
                        (_, true) => String::from(
                            "pass the two views as separate parameters rather than bundling \
                             them; or copy the shorter-lived data into an owned field",
                        ),
                        (Some(name), _) => format!(
                            "end the borrow before this: `{name}` is what keeps it alive, so \
                             shorten its last use or put it in a block of its own"
                        ),
                        (None, _) => format!(
                            "bind the borrow of `{name}` to a local and finish with it before \
                             this line, or copy the value out first",
                            name = name
                        ),
                    })
                    .note(if bundled {
                        "a view struct has one region: the intersection of its fields' (LT-2)"
                    } else {
                        "a borrow lasts until its last use, not to the end of the scope (BRW-2)"
                    }),
            );
        }
    }
}

/// `[DIA-3]` — which reference keeps this loan alive at a conflict, and where
/// it is next read.
///
/// The loan's region reaches a set of locals; the one to name is a local the
/// programmer wrote whose next read comes after the conflicting access, because
/// that read is the reason the borrow has not ended. A compiler temporary is
/// never named (`[DIA-2]`): the help talks about the expression instead.
fn keeper(
    body: &Body,
    regions: &Regions,
    reads: &HashMap<LocalId, Vec<Span>>,
    loan: &Loan,
    conflict: Span,
) -> (Option<String>, Option<Span>, Option<LocalId>) {
    let mut best: Option<(LocalId, Span)> = None;
    for holder in regions.holders(loan.region) {
        if body.local(*holder).name.is_none() {
            continue;
        }
        let Some(spans) = reads.get(holder) else { continue };
        for span in spans {
            if span.file != conflict.file || span.start <= conflict.start {
                continue;
            }
            if best.is_none_or(|(_, b)| span.start < b.start) {
                best = Some((*holder, *span));
            }
        }
    }
    match best {
        Some((holder, span)) => (body.local(holder).name.clone(), Some(span), Some(holder)),
        // No later read: the borrow is kept alive by something else — a loop
        // back edge, or the return slot — and the local it was written into is
        // still the honest thing to name.
        None => (body.local(loan.borrower).name.clone(), None, Some(loan.borrower)),
    }
}

/// `[LT-2]`, shape B13 — whether the thing keeping this loan alive is a view
/// struct bundling **more than one** view.
///
/// That is the whole of `[LT-2]`'s one-region model made visible: "constructing
/// a view struct from several references gives it the intersection of their
/// regions", and "multiple independent regions inside one struct are not
/// expressible in v1". So a struct holding two views holds *both* loans for as
/// long as any part of it is live, and touching the struct at all keeps the
/// shorter one alive — even where only the longer-lived field is ever read.
///
/// The rejection is right either way; what changes is the advice. `[DIA-7a]`
/// keys `E3064` to this shape, whose help is to stop bundling, and that is the
/// fix — where B3's "shorten its last use" is not, because the use that keeps
/// the loan alive may be of the *other* field entirely.
///
/// D-011 recorded `E3064` as "registered and emitted by nothing". It was
/// reachable all along: these programs were being rejected as B3.
fn bundles_two_views(body: &Body, types: &TypeTable, holder: Option<LocalId>) -> bool {
    let Some(holder) = holder else { return false };
    let TyKind::Struct(id) = *types.kind(body.local(holder).ty) else { return false };
    types.struct_def(id).fields.iter().filter(|f| types.is_view(f.ty)).count() >= 2
}

/// Every point at which a local's value is read, by the span of the statement
/// that reads it.
fn collect_reads(body: &Body) -> HashMap<LocalId, Vec<Span>> {
    let mut reads: HashMap<LocalId, Vec<Span>> = HashMap::new();
    let record = |accesses: Vec<(Place, Access)>, span: Span, reads: &mut HashMap<_, Vec<_>>| {
        for (place, access) in accesses {
            // A write through a reference reads the reference itself.
            let reading = matches!(access, Access::Read)
                || place.projection.iter().any(|p| matches!(p, Projection::Deref));
            if reading {
                reads.entry(place.local).or_default().push(span);
            }
        }
    };
    for block in &body.blocks {
        for stmt in &block.stmts {
            let mut accesses = Vec::new();
            match &stmt.kind {
                StmtKind::Assign { place, rvalue } => {
                    rvalue_reads(rvalue, &mut accesses);
                    accesses.push((place.clone(), Access::Write));
                }
                StmtKind::CheckedBinaryOp { lhs, rhs, .. } => {
                    operand_read(lhs, &mut accesses);
                    operand_read(rhs, &mut accesses);
                }
                StmtKind::Drop { place, .. } => accesses.push((place.clone(), Access::Read)),
                _ => {}
            }
            record(accesses, stmt.span, &mut reads);
        }
        let mut accesses = Vec::new();
        match &block.terminator {
            Terminator::SwitchInt { discr, .. } => operand_read(discr, &mut accesses),
            Terminator::Call { args, .. } => {
                for arg in args {
                    operand_read(arg, &mut accesses);
                }
            }
            Terminator::Assert { cond, .. } => operand_read(cond, &mut accesses),
            _ => {}
        }
        record(accesses, block.terminator_span, &mut reads);
    }
    reads
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
