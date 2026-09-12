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
    /// `[ARN-6]` — this mutable borrow was consumed by `Arena.scope` and is
    /// held by the returned `ScopedArena`. Conflicts use A1/E3096 rather than
    /// an ordinary B1/B3 alias diagnostic.
    arena_scope: bool,
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
    // `[BRW-4]` — a method's first parameter is its `self` receiver, so a
    // direct call to one of these bodies is a method call. Read once here
    // because the conflict is reported while checking the *caller's* body.
    let mut methods: HashSet<&str> = HashSet::new();
    for body in bodies {
        if is_method_body(body) {
            methods.insert(body.symbol.as_str());
        }
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
        FuncRef::Builtin {
            which:
                Builtin::SpanFrom { .. }
                | Builtin::ArenaAlloc { .. }
                | Builtin::FixedArenaAlloc { .. }
                | Builtin::ScopedArenaAlloc { .. }
                | Builtin::ArenaScope { .. },
            ..
        } => Elision::Named(vec![0]),
        // `[SPN-2]` — `get` and `get_unchecked` return a reference into the
        // view, so the view stays borrowed too.
        // A `str` built by `as_str()` points into its `String` the same way
        // a `Span` built by `as_span()` points into its container: without
        // this the view dangles across the next mutation (D-037).
        FuncRef::Builtin {
            which: Builtin::SpanGet | Builtin::SpanGetUnchecked | Builtin::StringAsStr,
            ..
        } => Elision::Named(vec![0]),
        FuncRef::Builtin { .. } => Elision::Nothing,
        // `[CLO-3]` — a call through a function value. `[EFF-2]`'s
        // reasoning applies to regions too: nothing is known about the
        // callee, so the permissive reading of `[LT-1]` rule 3 is taken
        // and the result is treated as borrowing every view argument.
        FuncRef::Indirect(_) => Elision::Everything,
    };
    let is_method = |func: &FuncRef| match func {
        FuncRef::Direct { symbol } => methods.contains(symbol.as_str()),
        _ => false,
    };
    for body in bodies {
        check_body(body, types, &elision, &is_method, sink);
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

/// `[BRW-4]` — whether this body is a method: its first parameter is the
/// `self` receiver. MIR keeps parameter names, so a caller recognises a
/// method call by its callee without trusting the mangled symbol.
fn is_method_body(body: &Body) -> bool {
    body.arg_count > 0 && body.local(LocalId(1)).name.as_deref() == Some("self")
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
    let is_method = |func: &FuncRef| match func {
        FuncRef::Direct { symbol } => symbol.as_str() == body.symbol.as_str() && is_method_body(body),
        _ => false,
    };
    check_body(body, types, &|_| Elision::Everything, &is_method, sink);
}

fn check_body(
    body: &Body,
    types: &TypeTable,
    elision: &dyn Fn(&FuncRef) -> Elision,
    is_method: &dyn Fn(&FuncRef) -> bool,
    sink: &mut Sink,
) {
    let live = liveness(body);
    let regions = Regions::infer(body, types, &live, elision);
    // Before the loans: a function that hands back a parameter has no loan of
    // its own, and `[LT-1a]` is about exactly that function.
    check_return_regions(body, types, &regions, sink);
    let loans = collect_loans(body, types, &regions, elision);
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
                    // A returned Arena allocation keeps the arena loan live
                    // through `Return`, so the compiler-generated arena drop
                    // at that boundary is the *manifestation* of `[LT-4]`, not
                    // a second ordinary alias error. `check_escapes` reports
                    // the required A1/E3061 shape below. User-visible reset
                    // and every non-returning drop remain ordinary writes.
                    if !(matches!(block.terminator, Terminator::Return)
                        && is_arena_ty(types, place_ty(body, types, place)))
                    {
                        accesses.push((place.clone(), Access::Write));
                    }
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
                is_method,
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
            Terminator::Assert { cond, msg, .. } => {
                operand_read(cond, &mut accesses);
                if let ember_mir::AssertKind::RefCellBorrow { file, line } = msg {
                    operand_read(file, &mut accesses);
                    operand_read(line, &mut accesses);
                }
            }
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
            is_method,
            sink,
            &mut reported,
        );

        // `[CELL-7]` — `L3011` fires when a `RefCell` guard is live across a
        // call that could re-enter the same cell. Conservative: any call while
        // a guard loan is live lints (a lint, never an error, and not opt-in).
        if let Terminator::Call { .. } = &block.terminator {
            check_refcell_call(body, types, &loans, &regions, point, block.terminator_span, sink);
        }

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
        if root.kind == LocalKind::Arg
            && (types.is_view(root.ty) || is_named_arena_origin(body, loan.place.local, types))
        {
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
        let is_arena = is_arena_ty(types, root.ty);
        if is_arena {
            let (origin_label, help, note) = if root.kind == LocalKind::Arg {
                (
                    format!(
                        "`{owner}` is a parameter, but the signature does not tie the return to it"
                    ),
                    format!(
                        "write `@borrows({owner})` above the wrapper, or return an owned value"
                    ),
                    "an Arena parameter is a return-provenance source only when `@borrows` names it (LT-4a)",
                )
            } else {
                (
                    format!("`{owner}` is local to this frame"),
                    "move the `Arena` to an outer scope, or copy the value out before it is reset"
                        .to_string(),
                    "Arena allocation views carry the region of the arena borrow (LT-4, ARN-1)",
                )
            };
            sink.emit_classified(
                Diagnostic::error(
                    codes::E3061,
                    span,
                    format!("arena allocation cannot outlive `{owner}`"),
                )
                .primary_label("this allocation view escapes the arena's region")
                .secondary(loan.span, format!("`{owner}` is borrowed for the allocation here"))
                .secondary(root.span, origin_label)
                .help(help)
                .note(note),
            );
        } else {
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
}

/// `[LT-4a]` — a growing Arena remains a non-view type, but an explicit
/// `@borrows(arena)` contract makes that parameter a valid return-provenance
/// origin for storage allocated from it. Fixed/scoped arenas are not inferred
/// into the exception: the owner named `Arena` specifically.
fn is_named_arena_origin(body: &Body, local: LocalId, types: &TypeTable) -> bool {
    if !is_growing_arena_ty(types, body.local(local).ty) || local.0 == 0 {
        return false;
    }
    body.borrows
        .as_ref()
        .is_some_and(|named| named.contains(&((local.0 - 1) as usize)))
}

fn is_growing_arena_ty(types: &TypeTable, ty: Ty) -> bool {
    matches!(types.kind(ty), TyKind::Struct(id) if types.struct_def(*id).name.as_str() == "Arena")
}

fn is_arena_ty(types: &TypeTable, ty: Ty) -> bool {
    matches!(types.kind(ty), TyKind::Struct(id)
        if matches!(types.struct_def(*id).name.as_str(), "Arena" | "FixedArena" | "ScopedArena"))
}

/// `[CELL-7]` — `L3011 RefCell guard held across a call`.
///
/// Fires when a `RefCell` guard loan is live across a function call that could
/// re-enter the same cell. Conservative and fail-closed: any call while a
/// guard is live lints, since the callee could reach the cell through a
/// parameter, a captured view, or recursion. A lint, never an error, and not
/// opt-in (`LNT-CFG-1` is about `[LT-1b]`'s `L3014`, a different lint).
fn check_refcell_call(
    body: &Body,
    types: &TypeTable,
    loans: &[Loan],
    regions: &Regions,
    point: Point,
    span: Span,
    sink: &mut Sink,
) {
    // One lint per call, naming the first live guard loan (by creation order
    // for determinism). `in_scope` is already live-loan-filtered via regions.
    // Only guard loans (borrows of a `RefCell`'s `value`) count: a whole-cell
    // borrow taken for the call's own `mut` argument is not a guard live
    // across the call.
    let mut live: Vec<&Loan> = in_scope(loans, regions, point)
        .into_iter()
        .filter(|loan| is_guard_loan(body, types, &loan.place))
        .collect();
    live.sort_by_key(|loan| (loan.created_at.block, loan.created_at.index));
    let Some(loan) = live.first() else {
        return;
    };
    // Name the cell, not its private `value` field: the loan is of field 0,
    // which no source ever writes. Strip one trailing `Field(0)` where the
    // parent is the cell for the display; the loan itself is unchanged.
    let mut display = loan.place.clone();
    if let Some(Projection::Field(0)) = display.projection.last() {
        let mut parent = display.clone();
        parent.projection.pop();
        if is_refcell_ty(types, place_ty(body, types, &parent)) {
            display = parent;
        }
    }
    let cell = place_name(body, types, &display);
    sink.emit(Diagnostic::lint(
        codes::L3011,
        span,
        format!("a `RefCell` guard for `{cell}` is live across this call"),
    )
    .primary_label("call happens here")
    .secondary(loan.span, "guard borrowed here")
    .help("drop the guard before the call, or put the call in a block of its own")
    .note("a guard live across a call that re-enters the same cell panics at run time [CELL-7]"));
}

/// Every explicit `Rvalue::Ref` in the body, plus `[LT-4a]`'s narrow
/// signature-level borrow of a non-view Arena argument. Default-mode value
/// parameters currently retain a by-value ABI representation, so the latter
/// has no `Rvalue::Ref`; recording it here keeps semantic provenance explicit
/// in the borrow analysis without pretending that every value parameter is a
/// view.
fn collect_loans(
    body: &Body,
    types: &TypeTable,
    regions: &Regions,
    elision: &dyn Fn(&FuncRef) -> Elision,
) -> Vec<Loan> {
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
                arena_scope: borrower_feeds_arena_scope(body, place.local),
            });
        }

        let Terminator::Call { func, args, dest, .. } = &block.terminator else {
            continue;
        };
        let Some(region) = regions.local_region(dest.local) else {
            continue;
        };
        let tied = elision(func);
        for (argument, operand) in args.iter().enumerate() {
            if !tied.ties(argument) {
                continue;
            }
            let (Operand::Copy(borrowed) | Operand::Move(borrowed)) = operand else {
                continue;
            };
            if !is_growing_arena_ty(types, place_ty(body, types, borrowed)) {
                continue;
            }
            let created_at = Point { block: block_index, index: block.stmts.len() };
            loans.push(Loan {
                place: borrowed.clone(),
                mutable: false,
                borrower: dest.local,
                region,
                created_at,
                span: block.terminator_span,
                reserved_at: HashSet::new(),
                arena_scope: false,
            });
        }
    }
    loans
}

fn borrower_feeds_arena_scope(body: &Body, borrower: LocalId) -> bool {
    body.blocks.iter().any(|block| {
        let Terminator::Call {
            func: FuncRef::Builtin { which: Builtin::ArenaScope { .. }, .. },
            args,
            ..
        } = &block.terminator
        else {
            return false;
        };
        args.iter().any(|arg| match arg {
            Operand::Copy(place) | Operand::Move(place) => place.local == borrower,
            Operand::Const(_) => false,
        })
    })
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
    // `[BRW-3]` — a reservation is a borrow taken *for* a call's receiver or
    // a `mut` argument: its borrower is always a temporary the lowering
    // created during argument evaluation. A borrow bound to a user local is a
    // full borrow from creation. Treating it as reserved merely because the
    // local is later used as a call argument downgrades genuine mutable
    // conflicts: two `ref mut` borrows reported B3/`E3021` instead of
    // B1/`E3022` (D-039 — found by probing, not by a test failing).
    if body.local(borrower).kind == LocalKind::User {
        return HashSet::new();
    }
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

/// `[BRW-4]`, shape B8 — the place a method call takes, when the conflict at
/// `point` is its receiver autoref.
///
/// A `mut self` call lowers to `tmp = &mut place` (borrower: a temporary the
/// lowering created) with the consuming call later whenever other arguments
/// need evaluating first. The walk below is `reservation_window`'s forward
/// search answering provenance instead of a window: the temporary's first use
/// must be a call's receiver, and the callee must be a method body. Anything
/// else — reassigned, read by another statement, fed to a free function or a
/// builtin — is not B8, and the conflict keeps its ordinary code.
fn method_autoref_target(
    body: &Body,
    point: Point,
    is_method: &dyn Fn(&FuncRef) -> bool,
) -> Option<Place> {
    let stmt = body.blocks.get(point.block)?.stmts.get(point.index)?;
    let StmtKind::Assign { place: borrower, rvalue: Rvalue::Ref { place: borrowed, mutable: true } } =
        &stmt.kind
    else {
        return None;
    };
    if !borrower.projection.is_empty() || body.local(borrower.local).kind != LocalKind::Temp {
        return None;
    }
    let borrower = borrower.local;
    let borrowed = borrowed.clone();
    let mut block_index = point.block;
    let mut start = point.index + 1;

    // Bounded like `reservation_window`: argument evaluation is a chain, and
    // a loop back would mean the temporary is used more than once anyway.
    for _ in 0..body.blocks.len() {
        let block = body.blocks.get(block_index)?;

        for stmt in block.stmts.iter().skip(start) {
            match &stmt.kind {
                StmtKind::Assign { place, rvalue } => {
                    if place.local == borrower {
                        return None;
                    }
                    let mut reads = Vec::new();
                    rvalue_reads(rvalue, &mut reads);
                    if reads.iter().any(|(p, _)| p.local == borrower) {
                        return None;
                    }
                }
                StmtKind::CheckedBinaryOp { lhs, rhs, .. } => {
                    let mut reads = Vec::new();
                    operand_read(lhs, &mut reads);
                    operand_read(rhs, &mut reads);
                    if reads.iter().any(|(p, _)| p.local == borrower) {
                        return None;
                    }
                }
                StmtKind::Drop { place, .. } => {
                    if place.local == borrower {
                        return None;
                    }
                }
                _ => {}
            }
        }

        match &block.terminator {
            Terminator::Call { func, args, next, .. } => {
                let receiver = args.first().and_then(|arg| match arg {
                    Operand::Copy(p) | Operand::Move(p) => Some(p.local),
                    Operand::Const(_) => None,
                });
                if receiver == Some(borrower) {
                    // The temporary is this call's receiver: B8 exactly when
                    // the callee is a method.
                    return if is_method(func) { Some(borrowed) } else { None };
                }
                if args.iter().any(|arg| match arg {
                    Operand::Copy(p) | Operand::Move(p) => p.local == borrower,
                    Operand::Const(_) => false,
                }) {
                    // Consumed somewhere other than the receiver: not B8.
                    return None;
                }
                block_index = next.0 as usize;
                start = 0;
            }
            Terminator::Assert { next, .. } => {
                block_index = next.0 as usize;
                start = 0;
            }
            Terminator::Goto(next) => {
                block_index = next.0 as usize;
                start = 0;
            }
            // A branch (or return) means the temporary outlives argument
            // evaluation: not the autoref shape, so keep the ordinary code.
            _ => return None,
        }
    }
    None
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
    is_method: &dyn Fn(&FuncRef) -> bool,
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
            // `[CELL-5]` — a guard's loan (a borrow of a `RefCell`'s `value`
            // field) never conflicts statically with another guard's borrow:
            // overlapping `borrow`/`borrow_mut` are allowed here and refused
            // by the counter at run time in every profile (`[CELL-9]`). Only
            // guard-vs-guard borrows are skipped: a whole-cell borrow (e.g. a
            // `mut` argument taking the cell for a call) must still conflict,
            // since moving the cell while a guard is live dangles it. Reads
            // and writes still conflict (see below). See `is_guard_loan`.
            if is_guard_loan(body, types, &loan.place) {
                if let Access::Borrow { .. } = access {
                    if is_guard_loan(body, types, place) {
                        continue;
                    }
                }
                // A shared guard loan still forbids moving the cell: an
                // ordinary shared loan allows `Read` (a move is a read), but
                // moving inline storage out from under a guard dangles it, so
                // reads conflict here too. (A mutable guard loan already
                // forbids reads.) Writes already conflict for every loan.
                if matches!(access, Access::Read) && !loan.mutable {
                    // Fall through to report below (via `refcell` flag).
                } else if matches!(access, Access::Read) {
                    // Mutable case already conflicts via `loan_mutable` below;
                    // keep the same path for the message.
                }
            }
            // `[BRW-3]` — inside the reservation window the borrow is not yet
            // mutable, so shared borrows and reads of the same place pass.
            let reserved = loan.mutable && loan.reserved_at.contains(&point);
            let loan_mutable = loan.mutable && !reserved;
            let refcell = is_guard_loan(body, types, &loan.place);
            let conflict = match access {
                // While a mutable borrow is live the owner may not read. A
                // shared `RefCell` loan also forbids reads: a move is a read,
                // and moving inline storage out from under a guard dangles it.
                Access::Read => loan_mutable || refcell,
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
            // `[BRW-4]` — when the conflicting access is a method call's
            // receiver autoref, disjoint-field access is defeated by the call:
            // shape B8, which `[DIA-7a]` keys to `E3025` (D-040). A free
            // function's `mut` argument lowers through the same temporary, so
            // the consuming call must be to a method body — otherwise a
            // whole-place `mut` argument misreports as B8 rather than B1.
            let method_root = if !bundled && loan_mutable && matches!(access, Access::Borrow { mutable: true }) {
                method_autoref_target(body, point, is_method)
                    .map(|taken| place_name(body, types, &taken))
            } else {
                None
            };
            let (code, message) = if loan.arena_scope {
                (codes::E3096, format!("`{name}` is scoped here"))
            } else if bundled {
                (
                    codes::E3064,
                    format!("`{name}` is borrowed through a view struct that bundles two views"),
                )
            } else if let Some(taken) = method_root.as_deref() {
                (
                    codes::E3025,
                    format!("cannot call a method on `{taken}` while `{name}` is mutably borrowed"),
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
                    .help(match (&borrower, loan.arena_scope, bundled, method_root.as_deref()) {
                        (_, true, _, _) => String::from(
                            "allocate from `scope` instead, or take this allocation before opening the scope",
                        ),
                        // `[DIA-7a]` shape B13's help, which is a different fix
                        // from B3's: the use keeping the loan alive may be of
                        // the *other* field, so shortening it is no answer.
                        (_, _, true, _) => String::from(
                            "pass the two views as separate parameters rather than bundling \
                             them; or copy the shorter-lived data into an owned field",
                        ),
                        // `[DIA-7a]` shape B8's help: the call takes all of
                        // `self`, so the fix is structural, not a shorter borrow.
                        (_, _, _, Some(_)) => String::from(
                            "inline the field access, take the two fields as separate \
                             parameters, or split the method",
                        ),
                        (Some(name), _, _, _) => format!(
                            "end the borrow before this: `{name}` is what keeps it alive, so \
                             shorten its last use or put it in a block of its own"
                        ),
                        (None, _, _, _) => format!(
                            "bind the borrow of `{name}` to a local and finish with it before \
                             this line, or copy the value out first",
                            name = name
                        ),
                    })
                    .note(if loan.arena_scope {
                        "a ScopedArena holds its parent's mutable borrow for the scope's whole region (ARN-6)"
                    } else if bundled {
                        "a view struct has one region: the intersection of its fields' (LT-2)"
                    } else if method_root.is_some() {
                        "a method takes all of `self`, so disjoint fields do not stay disjoint across a call (BRW-4)"
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
            Terminator::Assert { cond, msg, .. } => {
                operand_read(cond, &mut accesses);
                if let ember_mir::AssertKind::RefCellBorrow { file, line } = msg {
                    operand_read(file, &mut accesses);
                    operand_read(line, &mut accesses);
                }
            }
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
            // `[BRW-4]` — distinct fields of a struct or tuple are distinct
            // places, "through arbitrary nesting of field projections".
            (Projection::Field(x), Projection::Field(y)) if x != y => return false,
            // `[BRW-5]` — "unless both indices are constants and different".
            (Projection::ConstIndex(x), Projection::ConstIndex(y)) if x != y => return false,
            // A field beside an index is the compiler's own header access,
            // never the programmer's. `Array[T]` and `Span[T]` are `{ptr, len,
            // cap}` internally, and `lower_index` reads `len` through
            // `Field(1)` to bounds-check — but Ember has no `v.len` *field*,
            // only a `len()` method, so this pair can arise no other way.
            //
            // Treating it as an overlap made `[BRW-5]`'s exemption unreachable
            // for the container people actually use: `ref mut v[0]` and
            // `ref mut v[1]` were rejected because the second one's bounds
            // check "read `v`" while the first was live.
            //
            // This does not weaken anything a user can reach. `push` and the
            // rest take `mut v` — projection empty — which overlaps every
            // place under `v`, so a reallocation that would dangle an element
            // borrow is still caught.
            (Projection::Field(_), Projection::Index(_) | Projection::ConstIndex(_))
            | (Projection::Index(_) | Projection::ConstIndex(_), Projection::Field(_)) => {
                return false
            }
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

/// The type of a place, following its projections. A projection that does not
/// apply leaves the type alone (the type checker has already rejected such a
/// program; the borrow checker only has to stay on its feet).
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
            (Projection::Field(i), TyKind::Tuple(items)) => {
                ty = items.get(*i).copied().unwrap_or(ty);
            }
            (
                Projection::Index(_) | Projection::ConstIndex(_) | Projection::Column(_),
                TyKind::Array { elem, .. } | TyKind::Vec { elem } | TyKind::Span { elem, .. },
            ) => ty = *elem,
            (Projection::Deref, TyKind::Ref { inner, .. }) => ty = *inner,
            (Projection::Deref, TyKind::Ptr { inner, .. }) => ty = *inner,
            _ => {}
        }
    }
    ty
}

/// Whether a struct type is a compiler-known `RefCell[T]` (by name prefix,
/// like `is_option`'s `Option_` check in typeck).
fn is_refcell_ty(types: &TypeTable, ty: Ty) -> bool {
    match types.kind(ty) {
        TyKind::Struct(id) => types.struct_def(*id).name.as_str().starts_with("RefCell_"),
        _ => false,
    }
}

/// Whether this loan is a guard's loan: a borrow of a `RefCell`'s `value`
/// field (parent is a `RefCell`). Whole-cell borrows (e.g. a `mut` argument
/// taking the cell for a call) are `RefCell` loans but not guard loans: they
/// must still conflict statically (moving the cell while a guard is live
/// dangles it), and they must not trigger `L3011` on their own call.
fn is_guard_loan(body: &Body, types: &TypeTable, place: &Place) -> bool {
    if place.projection.is_empty() {
        return false;
    }
    let mut parent = place.clone();
    parent.projection.pop();
    is_refcell_ty(types, place_ty(body, types, &parent))
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
        Terminator::Assert { cond, msg, .. } => {
            let mut reads = Vec::new();
            operand_read(cond, &mut reads);
            if let ember_mir::AssertKind::RefCellBorrow { file, line } = msg {
                operand_read(file, &mut reads);
                operand_read(line, &mut reads);
            }
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
