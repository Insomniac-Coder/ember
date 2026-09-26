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
    AggregateKind, BasicBlock, BasicBlockId, Body, Builtin, CallableAccessSummary as CallAccessContract,
    CallableRegionMetadata, FuncRef,
    LocalId, LocalKind, Operand, ParameterFieldAccess, ParameterMode, Place, Projection, RegionAccessKind,
    ResultFieldProvenance, ResultProvenanceSummary, ResultRegionSource, Rvalue, Stmt, StmtKind,
    Terminator,
};
use ember_span::Span;
use ember_types::{StructId, Ty, TyKind, TypeTable};

use crate::facts::{
    AccessPermission, BorrowCapability, EscapeConstraint, ProvenanceRoot, ReferenceKind, StorageIdentity,
};
use crate::regions::{CallRegionContract, CallResultContract, Elision, Origin, Point, RegionVid, Regions};

/// One borrow, recorded where it is created (`[GLOSSARY]` "loan").
#[derive(Clone, Debug)]
struct Loan {
    /// `[IMP-7]` — the shared semantic facts for the borrowed storage. Access
    /// checking, region liveness, and diagnostics consume this one record
    /// rather than maintaining parallel place/mutability/region truths.
    capability: BorrowCapability,
    /// The local the reference was stored in. Named in the help line, and
    /// exempt from being its own conflicting access.
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
    /// `[ARN-6]` — this mutable borrow was consumed by `Arena.scope` and is
    /// held by the returned `ScopedArena`. Conflicts use A1/E3096 rather than
    /// an ordinary B1/B3 alias diagnostic.
    arena_scope: bool,
}

/// How a place is touched at a point (§4.7 step 5).
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum Access {
    Read,
    /// Ownership leaves this place. A move of a drop-owning value while a
    /// reference into it remains live can end the referent's lifetime even
    /// though the generated C expression looks like an ordinary read.
    Move,
    Write,
    Borrow {
        mutable: bool,
    },
}

pub fn check_all(bodies: &mut [Body], types: &TypeTable, sink: &mut Sink) {
    install_callable_regions_all(bodies, types);
    check_all_with_installed_callable_regions(bodies, types, sink);
}

/// `[MIR-REG-1]` — derive and install the canonical callable contract before
/// any caller consumes it. The driver may serialize/deserialize the installed
/// records at an interface-artifact boundary before invoking
/// [`check_all_with_installed_callable_regions`]; this producer stays separate
/// so a cache artifact cannot be confused with a second inference table.
pub fn install_callable_regions_all(bodies: &mut [Body], types: &TypeTable) {
    // `[LT-1]` is a property of the *callee's* signature, so a call in one
    // body is read against another's. The table is built once.
    let mut signatures: HashMap<String, Elision> = HashMap::new();
    for body in &*bodies {
        signatures.insert(body.symbol.clone(), elision_of(body, types));
    }
    let inferred = infer_callable_summaries(bodies, types, &signatures);
    for body in &mut *bodies {
        let contract = inferred
            .get(&body.symbol)
            .expect("callable summary inference omitted a MIR body");
        body.callable_regions = Some(metadata_from_contract(contract));
    }
}

/// Consume only the callable-region metadata already installed on MIR. A
/// missing or corrupt record is an internal compiler failure: `[LT-40]` does
/// not permit treating stale metadata as an optimisation hint or silently
/// widening it to an unknown call.
pub fn check_all_with_installed_callable_regions(
    bodies: &[Body],
    types: &TypeTable,
    sink: &mut Sink,
) {
    let signatures: HashMap<String, Elision> = bodies
        .iter()
        .map(|body| (body.symbol.clone(), elision_of(body, types)))
        .collect();
    // `[BRW-4]` — a method's first parameter is its `self` receiver, so a
    // direct call to one of these bodies is a method call. Read once here
    // because the conflict is reported while checking the *caller's* body.
    let methods: HashSet<String> = bodies
        .iter()
        .filter(|body| is_method_body(body))
        .map(|body| body.symbol.clone())
        .collect();
    // `[CLO-4]` is decided at a call boundary by both the callee's declared
    // parameter mode and the concrete closure environment's capture mode.
    // Keep those facts in MIR rather than guessing from a generated symbol or
    // treating all `owned` values as escaping views.
    let direct_param_modes: HashMap<String, Vec<ParameterMode>> = bodies
        .iter()
        .map(|body| (body.symbol.clone(), body.param_modes.clone()))
        .collect();
    let borrowing_closure_environments: HashSet<StructId> = bodies
        .iter()
        .filter(|body| !body.closure_captures_by_move)
        .filter_map(|body| body.closure_environment)
        .collect();
    // Borrow checking consumes the installed MIR/interface metadata, not the
    // temporary inference table. That makes the artifact a real producer /
    // consumer boundary rather than a duplicate cache beside the analysis.
    let summaries = contracts_from_metadata(bodies, &signatures);
    let call_contract = |func: &FuncRef| contract_for(func, &summaries, &signatures);
    let capture_contracts = closure_capture_contracts(bodies, &summaries);
    let owned_closure_environments: HashSet<StructId> = bodies
        .iter()
        .filter(|body| body.closure_captures_by_move)
        .filter_map(|body| body.closure_environment)
        .collect();
    let invalid_result_bodies =
        infer_invalid_result_bodies(bodies, types, &call_contract, &capture_contracts);
    let is_method = |func: &FuncRef| match func {
        FuncRef::Direct { symbol, .. } => methods.contains(symbol.as_str()),
        FuncRef::Interface { .. } | FuncRef::Virtual { .. } => true,
        _ => false,
    };
    for body in &*bodies {
        let capture_paths = capture_borrow_paths(body, &capture_contracts);
        check_body(
            body,
            types,
            &call_contract,
            &capture_paths,
            &owned_closure_environments,
            &borrowing_closure_environments,
            &direct_param_modes,
            &invalid_result_bodies,
            &is_method,
            sink,
        );
    }
}

/// `[HEAP-4]`, `[HEAP-5]`, `[RC-5]`, `[EXC-1]` — materialize the dynamic
/// reader/writer interval of a reference returned by `Shared.get()` or
/// `Shared.get_mut()`.
///
/// The type checker lowers the source operation to an ordinary `ref` of the
/// private payload projection. That keeps the static borrow proof and owner
/// liveness in the established machinery. This pass runs *after* that proof,
/// finds those compiler-private projections, and brackets the exact NLL region
/// with the existing runtime access operations on the owner handle. It is
/// deliberately an analysis transform rather than a codegen shortcut so MIR
/// verification, safety reporting, and C emission observe the same interval.
pub fn insert_shared_accesses_all(bodies: &mut [Body], types: &TypeTable) -> usize {
    let signatures: HashMap<String, Elision> = bodies
        .iter()
        .map(|body| (body.symbol.clone(), elision_of(body, types)))
        .collect();
    let summaries = contracts_from_metadata(bodies, &signatures);
    let call_contract = |func: &FuncRef| contract_for(func, &summaries, &signatures);
    let capture_contracts = closure_capture_contracts(bodies, &summaries);
    let (returned_accesses, returning_accesses) = infer_shared_return_accesses(
        bodies,
        types,
        &call_contract,
        &capture_contracts,
    );

    bodies
        .iter_mut()
        .map(|body| {
            let capture_paths = capture_borrow_paths(body, &capture_contracts);
            let regions = Regions::infer_with_capture_borrow_paths(
                body,
                types,
                &call_contract,
                &capture_paths,
            );
            insert_shared_accesses(
                body,
                types,
                &regions,
                &returned_accesses,
                &returning_accesses,
            )
        })
        .sum()
}

/// A dynamic `Shared.get()`/`Shared.get_mut()` interval that crosses a direct call. This is
/// kept separate from callable-region metadata: it affects compiler-internal
/// runtime instrumentation, not the public borrow/ABI contract.
#[derive(Clone, PartialEq, Eq)]
struct SharedReturnAccess {
    argument: usize,
    /// Whether the callee receives this source through the ordinary `mut`
    /// parameter ABI (`ref mut T`) rather than directly by value.
    argument_is_mut_ref: bool,
    mutable: bool,
    /// Projections from the caller's mutable argument place to the
    /// `Shared[_]` owner. The empty path is the ordinary `mut owner:
    /// Shared[T]` case.
    owner_projection: Vec<Projection>,
}

/// Infer the direct-call transfer source for a returned `Shared.get()` or `Shared.get_mut()`
/// reference. A wrapper can return another direct wrapper, so this closes to
/// a fixpoint. Multiple possible owner sources deliberately receive no
/// *singular* summary: the reference ABI has no hidden owner word, so the
/// callee starts an exact per-owner transfer and the caller closes it from the
/// returned payload instead. The ordinary region checker still validates the
/// source program independently.
fn infer_shared_return_accesses(
    bodies: &[Body],
    types: &TypeTable,
    call_contract: &dyn Fn(&FuncRef) -> CallRegionContract,
    capture_contracts: &HashMap<StructId, CallAccessContract>,
) -> (HashMap<String, SharedReturnAccess>, HashSet<String>) {
    let mut summaries = HashMap::new();
    let mut returning = HashSet::new();
    for _ in 0..=bodies.len() {
        let mut next = HashMap::new();
        let mut next_returning = HashSet::new();
        for body in bodies {
            let capture_paths = capture_borrow_paths(body, capture_contracts);
            let regions = Regions::infer_with_capture_borrow_paths(
                body,
                types,
                call_contract,
                &capture_paths,
            );
            let (summary, returns_shared_access) =
                shared_return_access(body, types, &regions, &summaries, &returning);
            if returns_shared_access {
                next_returning.insert(body.symbol.clone());
            }
            if let Some(summary) = summary {
                next.insert(body.symbol.clone(), summary);
            }
        }
        if next == summaries && next_returning == returning {
            return (summaries, returning);
        }
        summaries = next;
        returning = next_returning;
    }
    (summaries, returning)
}

fn shared_return_access(
    body: &Body,
    types: &TypeTable,
    regions: &Regions,
    summaries: &HashMap<String, SharedReturnAccess>,
    returning: &HashSet<String>,
) -> (Option<SharedReturnAccess>, bool) {
    let mut candidates = Vec::new();
    let mut returns_shared_access = false;

    for (block, basic_block) in body.blocks.iter().enumerate() {
        for (index, stmt) in basic_block.stmts.iter().enumerate() {
            let StmtKind::Assign {
                rvalue: Rvalue::Ref { place, mutable },
                ..
            } = &stmt.kind
            else {
                continue;
            };
            let Some(owner) = shared_owner_of_payload(body, types, place) else {
                continue;
            };
            let point = Point { block, index };
            let Some(region) = regions.loan_region(point) else {
                continue;
            };
            if region_reaches_return(body, regions, region) {
                returns_shared_access = true;
                if let Some(candidate) = shared_return_owner_argument(body, &owner, *mutable) {
                    candidates.push(candidate);
                }
            }
        }

        let Terminator::Call { func: FuncRef::Direct { symbol, .. }, args, dest, .. } =
            &basic_block.terminator
        else {
            continue;
        };
        if !returning.contains(symbol.as_str()) {
            continue;
        }
        let Some(region) = regions.local_region(dest.local) else {
            continue;
        };
        if !region_reaches_return(body, regions, region) {
            continue;
        }
        returns_shared_access = true;
        let Some(summary) = summaries.get(symbol.as_str()) else {
            continue;
        };
        let Some(argument) = args.get(summary.argument) else {
            continue;
        };
        let Some(mut owner) =
            shared_argument_source(body, types, argument, summary.argument_is_mut_ref)
        else {
            continue;
        };
        owner.projection.extend(summary.owner_projection.iter().cloned());
        if is_shared_owner(body, types, &owner)
            && let Some(candidate) = shared_return_owner_argument(body, &owner, summary.mutable)
        {
            candidates.push(candidate);
        }
    }

    candidates.dedup();
    ((candidates.len() == 1).then(|| candidates.remove(0)), returns_shared_access)
}

fn shared_return_owner_argument(
    body: &Body,
    owner: &Place,
    mutable: bool,
) -> Option<SharedReturnAccess> {
    if body.local(owner.local).kind != LocalKind::Arg {
        return None;
    }
    let argument_is_mut_ref = matches!(owner.projection.first(), Some(Projection::Deref));
    let owner_projection = if argument_is_mut_ref {
        owner.projection[1..].to_vec()
    } else {
        owner.projection.clone()
    };
    Some(SharedReturnAccess {
        argument: owner.local.0.checked_sub(1)? as usize,
        argument_is_mut_ref,
        mutable,
        owner_projection,
    })
}

fn region_reaches_return(body: &Body, regions: &Regions, region: RegionVid) -> bool {
    body.blocks.iter().enumerate().any(|(block, candidate)| {
        matches!(candidate.terminator, Terminator::Return)
            && regions.contains(
                region,
                Point {
                    block,
                    index: candidate.stmts.len(),
                },
            )
    })
}

/// Recover the source place of a temporary made for a `mut` argument. The
/// lowerer gives every ordinary `mut` argument this shape; requiring exactly
/// one definition keeps the runtime proof fail-closed across unusual MIR.
fn mutable_ref_source(body: &Body, operand: &Operand) -> Option<Place> {
    let (Operand::Copy(source) | Operand::Move(source)) = operand else {
        return None;
    };
    if !source.projection.is_empty() {
        return None;
    }
    let mut found = None;
    for block in &body.blocks {
        for stmt in &block.stmts {
            let StmtKind::Assign {
                place,
                rvalue: Rvalue::Ref { place: borrowed, mutable: true },
            } = &stmt.kind
            else {
                continue;
            };
            if place.local != source.local || !place.projection.is_empty() {
                continue;
            }
            if found.replace(borrowed.clone()).is_some() {
                return None;
            }
        }
    }
    found
}

fn shared_argument_source(
    body: &Body,
    types: &TypeTable,
    operand: &Operand,
    argument_is_mut_ref: bool,
) -> Option<Place> {
    if argument_is_mut_ref {
        return mutable_ref_source(body, operand);
    }
    let (Operand::Copy(place) | Operand::Move(place)) = operand else {
        return None;
    };
    is_shared_owner(body, types, place).then(|| place.clone())
}

#[derive(Clone)]
struct SharedAccess {
    created_at: Point,
    owner: Place,
    mutable: bool,
    region: RegionVid,
    span: Span,
    start: SharedAccessStart,
    end: SharedAccessEnd,
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum SharedAccessStart {
    Normal,
    Transfer,
    None,
}

#[derive(Copy, Clone)]
enum SharedAccessEnd {
    Normal,
    Transfer,
}

#[derive(Clone)]
enum AccessEvent {
    Begin { place: Place, mutable: bool, transfer: bool, span: Span },
    End { place: Place, mutable: bool, transfer: bool, span: Span },
}

impl AccessEvent {
    fn is_end(&self) -> bool {
        matches!(self, Self::End { .. })
    }

    fn into_stmt(self) -> Stmt {
        match self {
            Self::Begin { place, mutable, transfer: false, span } => {
                Stmt::new(StmtKind::BeginAccess { place, mutable }, span)
            }
            Self::Begin { place, mutable, transfer: true, span } => {
                Stmt::new(StmtKind::BeginAccessTransfer { place, mutable }, span)
            }
            Self::End { place, mutable, transfer: false, span } => {
                Stmt::new(StmtKind::EndAccess { place, mutable }, span)
            }
            Self::End { place, mutable, transfer: true, span } => {
                Stmt::new(StmtKind::EndAccessTransfer { place, mutable }, span)
            }
        }
    }
}

fn end_event(access: &SharedAccess) -> AccessEvent {
    AccessEvent::End {
        place: access.owner.clone(),
        mutable: access.mutable,
        transfer: matches!(access.end, SharedAccessEnd::Transfer),
        span: access.span,
    }
}

/// Add access brackets to one body without re-running the borrow checker.
/// Direct calls that return a Shared payload reference begin the transferred
/// reader/writer interval in their caller; ordinary local-reference shapes are
/// bracketed in their defining body. Both use exact NLL end points, including
/// branching CFG exits.
fn insert_shared_accesses(
    body: &mut Body,
    types: &TypeTable,
    regions: &Regions,
    returned_accesses: &HashMap<String, SharedReturnAccess>,
    returning_accesses: &HashSet<String>,
) -> usize {
    let original_blocks = body.blocks.len();
    let mut accesses = Vec::new();
    for (block, basic_block) in body.blocks.iter().enumerate() {
        for (index, stmt) in basic_block.stmts.iter().enumerate() {
            let StmtKind::Assign {
                rvalue: Rvalue::Ref { place, mutable },
                ..
            } = &stmt.kind
            else {
                continue;
            };
            let Some(owner) = shared_owner_of_payload(body, types, place) else {
                continue;
            };
            let created_at = Point { block, index };
            let Some(region) = regions.loan_region(created_at) else {
                continue;
            };
            let reaches_return = region_reaches_return(body, regions, region);
            // Singular return sources start in the caller, where the ordinary
            // owner place is available. A multi-source return begins in this
            // callee and closes through the selected payload in its caller.
            if reaches_return && returned_accesses.contains_key(body.symbol.as_str()) {
                continue;
            }
            accesses.push(SharedAccess {
                created_at,
                owner,
                mutable: *mutable,
                region,
                span: stmt.span,
                start: if reaches_return {
                    SharedAccessStart::Transfer
                } else {
                    SharedAccessStart::Normal
                },
                end: SharedAccessEnd::Normal,
            });
        }
    }
    for (_block, basic_block) in body.blocks.iter().take(original_blocks).enumerate() {
        let Terminator::Call { func: FuncRef::Direct { symbol, .. }, args, dest, next } =
            &basic_block.terminator
        else {
            continue;
        };
        let Some(summary) = returned_accesses.get(symbol.as_str()) else {
            continue;
        };
        let Some(argument) = args.get(summary.argument) else {
            continue;
        };
        let Some(mut owner) =
            shared_argument_source(body, types, argument, summary.argument_is_mut_ref)
        else {
            continue;
        };
        owner.projection.extend(summary.owner_projection.iter().cloned());
        if !is_shared_owner(body, types, &owner) {
            continue;
        }
        let Some(region) = regions.local_region(dest.local) else {
            continue;
        };
        // This body itself returns the reference, so its caller owns the
        // interval transfer. Opening it here would leak an access across this
        // frame's return boundary.
        if region_reaches_return(body, regions, region) {
            continue;
        }
        accesses.push(SharedAccess {
            created_at: Point { block: next.0 as usize, index: 0 },
            owner,
            mutable: summary.mutable,
            region,
            span: basic_block.terminator_span,
            start: SharedAccessStart::Normal,
            end: SharedAccessEnd::Normal,
        });
    }
    // A callee with more than one possible Shared owner starts the exact
    // access on its selected branch. Its caller cannot reconstruct that
    // selection from the reference ABI, but it can close the interval through
    // the returned payload's header at the reference's exact NLL end.
    for (_block, basic_block) in body.blocks.iter().take(original_blocks).enumerate() {
        let Terminator::Call { func: FuncRef::Direct { symbol, .. }, dest, next, .. } =
            &basic_block.terminator
        else {
            continue;
        };
        if returned_accesses.contains_key(symbol.as_str())
            || !returning_accesses.contains(symbol.as_str())
        {
            continue;
        }
        let Some(region) = regions.local_region(dest.local) else {
            continue;
        };
        if region_reaches_return(body, regions, region) {
            continue;
        }
        let TyKind::Ref { mutable, .. } = *types.kind(place_ty(body, types, dest)) else {
            continue;
        };
        let mut payload = dest.clone();
        payload.projection.push(Projection::Deref);
        accesses.push(SharedAccess {
            created_at: Point { block: next.0 as usize, index: 0 },
            owner: payload,
            mutable,
            region,
            span: basic_block.terminator_span,
            start: SharedAccessStart::None,
            end: SharedAccessEnd::Transfer,
        });
    }
    if accesses.is_empty() {
        return 0;
    }

    let mut inline: HashMap<(usize, usize), Vec<AccessEvent>> = HashMap::new();
    let mut edge_ends = HashMap::new();
    for access in &accesses {
        // An end at the same location as a new access has to execute first:
        // that is the NLL reuse case (`first` dies before `second` starts).
        if access.start != SharedAccessStart::None {
            inline
                .entry((access.created_at.block, access.created_at.index))
                .or_default()
                .push(AccessEvent::Begin {
                    place: access.owner.clone(),
                    mutable: access.mutable,
                    transfer: access.start == SharedAccessStart::Transfer,
                    span: access.span,
                });
        }

        for block in 0..original_blocks {
            let statement_count = body.blocks[block].stmts.len();
            for index in 0..statement_count {
                let point = Point { block, index };
                if !access_active(access, regions, point) {
                    continue;
                }
                let following = Point { block, index: index + 1 };
                if !access_active(access, regions, following) {
                    inline
                        .entry((block, index + 1))
                        .or_default()
                        .push(end_event(access));
                }
            }

            let terminal = Point { block, index: statement_count };
            if !access_active(access, regions, terminal) {
                continue;
            }
            let successors = access_successors(&body.blocks[block].terminator);
            if successors.is_empty()
                || successors.iter().all(|successor| {
                    !access_active(access, regions, Point { block: successor.0 as usize, index: 0 })
                })
            {
                if !(matches!(body.blocks[block].terminator, Terminator::Return)
                    && access.start == SharedAccessStart::Transfer)
                {
                    inline
                        .entry((block, statement_count))
                        .or_default()
                        .push(end_event(access));
                }
                continue;
            }
            for successor in successors {
                if access_active(
                    access,
                    regions,
                    Point { block: successor.0 as usize, index: 0 },
                ) {
                    continue;
                }
                append_edge_end(
                    body,
                    block,
                    successor,
                    end_event(access),
                    &mut edge_ends,
                );
            }
        }
    }

    for (block, basic_block) in body.blocks.iter_mut().take(original_blocks).enumerate() {
        let original = std::mem::take(&mut basic_block.stmts);
        let mut rewritten = Vec::with_capacity(original.len() + inline.len());
        let original_len = original.len();
        for (index, stmt) in original.into_iter().enumerate() {
            if let Some(events) = inline.get(&(block, index)) {
                for event in events.iter().filter(|event| event.is_end()) {
                    rewritten.push(event.clone().into_stmt());
                }
                for event in events.iter().filter(|event| !event.is_end()) {
                    rewritten.push(event.clone().into_stmt());
                }
            }
            rewritten.push(stmt);
        }
        if let Some(events) = inline.get(&(block, original_len)) {
            for event in events.iter().filter(|event| event.is_end()) {
                rewritten.push(event.clone().into_stmt());
            }
            for event in events.iter().filter(|event| !event.is_end()) {
                rewritten.push(event.clone().into_stmt());
            }
        }
        basic_block.stmts = rewritten;
    }
    accesses.len()
}

fn shared_owner_of_payload(body: &Body, types: &TypeTable, payload: &Place) -> Option<Place> {
    let (Projection::Field(0), Projection::Deref) =
        (payload.projection.get(payload.projection.len().checked_sub(2)?)?, payload.projection.last()?)
    else {
        return None;
    };
    let mut owner = payload.clone();
    owner.projection.truncate(owner.projection.len() - 2);
    is_shared_owner(body, types, &owner).then_some(owner)
}

fn is_shared_owner(body: &Body, types: &TypeTable, owner: &Place) -> bool {
    let TyKind::Struct(id) = types.kind(place_ty(body, types, owner)) else {
        return false;
    };
    types.struct_def(*id)
        .origin
        .as_ref()
        .is_some_and(|(name, args)| name.is("Shared") && args.len() == 1)
}

fn access_active(access: &SharedAccess, regions: &Regions, point: Point) -> bool {
    point == access.created_at || regions.contains(access.region, point)
}

fn access_successors(terminator: &Terminator) -> Vec<BasicBlockId> {
    match terminator {
        Terminator::Goto(target) => vec![*target],
        Terminator::SwitchInt { targets, otherwise, .. } => {
            let mut result: Vec<_> = targets.iter().map(|(_, target)| *target).collect();
            result.push(*otherwise);
            result.sort();
            result.dedup();
            result
        }
        Terminator::Call { next, .. } | Terminator::Assert { next, .. } => vec![*next],
        Terminator::Return | Terminator::Unreachable => Vec::new(),
    }
}

fn append_edge_end(
    body: &mut Body,
    source: usize,
    target: BasicBlockId,
    event: AccessEvent,
    edges: &mut HashMap<(usize, BasicBlockId), BasicBlockId>,
) {
    if let Some(existing) = edges.get(&(source, target)).copied() {
        body.blocks[existing.0 as usize].stmts.push(event.into_stmt());
        return;
    }
    let bridge = BasicBlockId(body.blocks.len() as u32);
    body.blocks.push(BasicBlock {
        stmts: vec![event.clone().into_stmt()],
        terminator: Terminator::Goto(target),
        terminator_span: match event {
            AccessEvent::End { span, .. } => span,
            AccessEvent::Begin { span, .. } => span,
        },
    });
    let terminator = &mut body.blocks[source].terminator;
    match terminator {
        Terminator::Goto(next) => {
            if *next == target {
                *next = bridge;
            }
        }
        Terminator::SwitchInt { targets, otherwise, .. } => {
            for (_, next) in targets {
                if *next == target {
                    *next = bridge;
                }
            }
            if *otherwise == target {
                *otherwise = bridge;
            }
        }
        Terminator::Call { next, .. } | Terminator::Assert { next, .. } => {
            if *next == target {
                *next = bridge;
            }
        }
        Terminator::Return | Terminator::Unreachable => {}
    }
    edges.insert((source, target), bridge);
}

fn metadata_from_contract(contract: &CallRegionContract) -> CallableRegionMetadata {
    CallableRegionMetadata::new(
        contract.access.clone(),
        match &contract.result {
            CallResultContract::Fields(summary) => Some(summary.clone()),
            CallResultContract::Legacy(_) => None,
        },
    )
}

fn contracts_from_metadata(
    bodies: &[Body],
    signatures: &HashMap<String, Elision>,
) -> HashMap<String, CallRegionContract> {
    bodies
        .iter()
        .map(|body| {
            let metadata = body
                .callable_regions
                .as_ref()
                .expect("borrow checking requires installed callable-region metadata");
            let result = metadata.result.clone().map_or_else(
                || {
                    CallResultContract::Legacy(
                        signatures
                            .get(body.symbol.as_str())
                            .cloned()
                            .unwrap_or(Elision::Everything),
                    )
                },
                CallResultContract::Fields,
            );
            (
                body.symbol.clone(),
                CallRegionContract {
                    access: metadata.access.clone(),
                    result,
                    latebound: false,
                },
            )
        })
        .collect()
}

/// `[LT-42]` — a capturing closure's environment type is compiler-generated
/// and carried explicitly by MIR. Its access contract is therefore a verified
/// source for narrowing the *synthetic* `&capture` used to construct that
/// environment. No user-written `ref T` is eligible for this treatment.
fn closure_capture_contracts(
    bodies: &[Body],
    contracts: &HashMap<String, CallRegionContract>,
) -> HashMap<StructId, CallAccessContract> {
    let mut result = HashMap::new();
    for body in bodies {
        let Some(environment) = body.closure_environment else { continue };
        let Some(contract) = contracts.get(&body.symbol) else { continue };
        // Each generated closure environment is nominally unique. Should an
        // invalid MIR producer ever reuse one, retain the conservative choice
        // by removing the identity instead of selecting either body.
        if let Some(previous) = result.insert(environment, contract.access.clone())
            && previous != contract.access
        {
            result.remove(&environment);
        }
    }
    result
}

/// Locate synthetic capture borrows in a closure creator. A borrow is exact
/// only when its temporary flows once, directly into a known generated
/// environment field and the closure body's verified summary reaches that
/// field through its `ref` dereference. Anything else stays an ordinary
/// whole-place borrow.
fn capture_borrow_paths(
    body: &Body,
    closure_contracts: &HashMap<StructId, CallAccessContract>,
) -> HashMap<Point, Vec<Vec<Projection>>> {
    let mut result = HashMap::new();
    for (block_index, block) in body.blocks.iter().enumerate() {
        for (index, stmt) in block.stmts.iter().enumerate() {
            let StmtKind::Assign {
                place: destination,
                rvalue: Rvalue::Ref { mutable: false, .. },
            } = &stmt.kind
            else {
                continue;
            };
            if !destination.projection.is_empty() || body.local(destination.local).kind != LocalKind::Temp {
                continue;
            }
            let uses = capture_temporary_uses(body, Point { block: block_index, index }, destination.local);
            let [CaptureTemporaryUse::Environment { environment, field }] = uses.as_slice() else {
                continue;
            };
            let Some(contract) = closure_contracts.get(environment) else { continue };
            let Some(paths) = capture_field_paths(contract, *field) else { continue };
            result.insert(Point { block: block_index, index }, paths);
        }
    }
    result
}

/// Every value flow from a candidate synthetic capture temporary. The precise
/// path is admissible only when this list has exactly one `Environment` item;
/// a second use, a redefinition, an indirect call, or any other flow makes the
/// creator fall back to the ordinary whole-place borrow.
enum CaptureTemporaryUse {
    Environment { environment: StructId, field: usize },
    Other,
}

fn capture_temporary_uses(
    body: &Body,
    created_at: Point,
    local: LocalId,
) -> Vec<CaptureTemporaryUse> {
    let mut uses = Vec::new();
    for (block_index, block) in body.blocks.iter().enumerate() {
        for (index, stmt) in block.stmts.iter().enumerate() {
            let point = Point { block: block_index, index };
            let StmtKind::Assign { place, rvalue } = &stmt.kind else { continue };
            if point != created_at && place.local == local {
                uses.push(CaptureTemporaryUse::Other);
            }
            match rvalue {
                Rvalue::Aggregate {
                    kind: ember_mir::AggregateKind::Struct(environment),
                    operands,
                } => {
                    for (field, operand) in operands.iter().enumerate() {
                        capture_operand_use(operand, local, || {
                            CaptureTemporaryUse::Environment {
                                environment: *environment,
                                field,
                            }
                        }, &mut uses);
                    }
                }
                _ => capture_rvalue_uses(rvalue, local, &mut uses),
            }
        }
        match &block.terminator {
            Terminator::SwitchInt { discr, .. } => {
                capture_operand_use(discr, local, || CaptureTemporaryUse::Other, &mut uses)
            }
            Terminator::Call { func, args, dest, .. } => {
                if dest.local == local {
                    uses.push(CaptureTemporaryUse::Other);
                }
                if let FuncRef::Indirect { operand, .. } = func {
                    capture_operand_use(operand, local, || CaptureTemporaryUse::Other, &mut uses);
                }
                for operand in args {
                    capture_operand_use(operand, local, || CaptureTemporaryUse::Other, &mut uses);
                }
            }
            Terminator::Assert { cond, msg, .. } => {
                capture_operand_use(cond, local, || CaptureTemporaryUse::Other, &mut uses);
                if let ember_mir::AssertKind::RefCellBorrow { file, line } = msg {
                    capture_operand_use(file, local, || CaptureTemporaryUse::Other, &mut uses);
                    capture_operand_use(line, local, || CaptureTemporaryUse::Other, &mut uses);
                }
                if let ember_mir::AssertKind::Panic { message } = msg {
                    capture_operand_use(message, local, || CaptureTemporaryUse::Other, &mut uses);
                }
            }
            Terminator::Goto(_) | Terminator::Return | Terminator::Unreachable => {}
        }
    }
    uses
}

fn capture_rvalue_uses(
    rvalue: &Rvalue,
    local: LocalId,
    uses: &mut Vec<CaptureTemporaryUse>,
) {
    let mut add = |operand: &Operand| {
        capture_operand_use(operand, local, || CaptureTemporaryUse::Other, uses)
    };
    match rvalue {
        Rvalue::Use(operand) | Rvalue::UnaryOp { operand, .. } | Rvalue::Cast { operand, .. } => {
            add(operand)
        }
        Rvalue::BinaryOp { lhs, rhs, .. } => {
            add(lhs);
            add(rhs);
        }
        Rvalue::Aggregate { operands, .. } => {
            for operand in operands {
                add(operand);
            }
        }
        Rvalue::Repeat { value, .. } => add(value),
        Rvalue::Discriminant(place) | Rvalue::Ref { place, .. }
            if place.local == local =>
        {
            uses.push(CaptureTemporaryUse::Other)
        }
        Rvalue::Discriminant(_) | Rvalue::Ref { .. } => {}
    }
}

fn capture_operand_use(
    operand: &Operand,
    local: LocalId,
    kind: impl FnOnce() -> CaptureTemporaryUse,
    uses: &mut Vec<CaptureTemporaryUse>,
) {
    if matches!(operand, Operand::Copy(place) | Operand::Move(place)
        if place.local == local && place.projection.is_empty())
    {
        uses.push(kind());
    }
}

fn capture_field_paths(
    contract: &CallAccessContract,
    field: usize,
) -> Option<Vec<Vec<Projection>>> {
    let CallAccessContract::Fields(accesses) = contract else { return None };
    let mut paths = Vec::new();
    for access in accesses {
        if access.argument != 0 {
            continue;
        }
        let [Projection::Field(capture), Projection::Deref, rest @ ..] = access.projection.as_slice()
        else {
            continue;
        };
        if *capture == field {
            // An empty suffix means the closure used its captured aggregate as
            // a whole value, which must retain every field.
            if rest.is_empty() {
                return None;
            }
            if !paths.iter().any(|path| path == rest) {
                paths.push(rest.to_vec());
            }
        }
    }
    (!paths.is_empty()).then_some(paths)
}

/// `[VERIFY-3]` — rederive callable metadata from MIR and compare it with the
/// artifact consumed by callers. A mismatch is an internal compiler failure,
/// not an optimization hint or a reason to widen a region to `static`.
pub fn verify_callable_regions_all(
    bodies: &[Body],
    types: &TypeTable,
) -> Vec<ember_mir::verify::Violation> {
    let signatures: HashMap<String, Elision> = bodies
        .iter()
        .map(|body| (body.symbol.clone(), elision_of(body, types)))
        .collect();
    let expected = infer_callable_summaries(bodies, types, &signatures);
    let mut violations = verify_closure_environments(bodies, types);
    for body in bodies {
        let Some(actual) = &body.callable_regions else {
            violations.push(ember_mir::verify::Violation {
                body: body.symbol.clone(),
                message: "callable-region metadata is missing".to_string(),
            });
            continue;
        };
        if !actual.fingerprint_is_valid() {
            violations.push(ember_mir::verify::Violation {
                body: body.symbol.clone(),
                message: "callable-region metadata fingerprint is stale or corrupt".to_string(),
            });
            continue;
        }
        let wanted = expected
            .get(&body.symbol)
            .expect("callable summary verification omitted a MIR body");
        if *actual != metadata_from_contract(wanted) {
            violations.push(ember_mir::verify::Violation {
                body: body.symbol.clone(),
                message: "callable-region metadata disagrees with the MIR body".to_string(),
            });
        }
    }
    if bodies.iter().all(|body| {
        body.callable_regions
            .as_ref()
            .is_some_and(CallableRegionMetadata::fingerprint_is_valid)
    }) {
        let installed = contracts_from_metadata(bodies, &signatures);
        let call_contract = |func: &FuncRef| contract_for(func, &installed, &signatures);
        let capture_contracts = closure_capture_contracts(bodies, &installed);
        for body in bodies {
            let capture_paths = capture_borrow_paths(body, &capture_contracts);
            let regions = Regions::infer_with_capture_borrow_paths(
                body,
                types,
                &call_contract,
                &capture_paths,
            );
            violations.extend(
                regions
                    .verify_call_contracts(body, types, &call_contract)
                    .into_iter()
                    .map(|message| ember_mir::verify::Violation {
                        body: body.symbol.clone(),
                        message,
                    }),
            );
        }
    }
    violations
}

/// The closure environment identity consumed by `[LT-42]` must describe the
/// actual first parameter of one and only one closure body. This turns a
/// malformed lowering record into a code-generation-boundary violation rather
/// than letting it manufacture a precise capture fact.
fn verify_closure_environments(
    bodies: &[Body],
    types: &TypeTable,
) -> Vec<ember_mir::verify::Violation> {
    let mut owners: HashMap<StructId, &Body> = HashMap::new();
    let mut violations = Vec::new();
    for body in bodies {
        if body.closure_captures_by_move && body.closure_environment.is_none() {
            violations.push(ember_mir::verify::Violation {
                body: body.symbol.clone(),
                message: "owned-capture marker has no closure environment".to_string(),
            });
            continue;
        }
        let Some(environment) = body.closure_environment else { continue };
        let parameter = (body.arg_count >= 1).then(|| body.local(LocalId(1)));
        if !parameter.is_some_and(|parameter| {
            match types.kind(parameter.ty) {
                TyKind::Struct(found) => *found == environment,
                TyKind::Ref { mutable: true, inner } => {
                    matches!(types.kind(*inner), TyKind::Struct(found) if *found == environment)
                }
                _ => false,
            }
        }) {
            violations.push(ember_mir::verify::Violation {
                body: body.symbol.clone(),
                message: "closure-environment identity does not match parameter 0".to_string(),
            });
            continue;
        }
        if let Some(previous) = owners.insert(environment, body) {
            violations.push(ember_mir::verify::Violation {
                body: body.symbol.clone(),
                message: format!(
                    "closure-environment identity is also owned by `{}`",
                    previous.symbol
                ),
            });
        }
    }
    violations
}

/// The pre-0.9.5 whole-result relation, retained both for one-region results
/// and as the mandatory conservative fallback for calls with no exact
/// field-to-source summary.
fn legacy_elision(func: &FuncRef, signatures: &HashMap<String, Elision>) -> Elision {
    match func {
        FuncRef::Direct { symbol, .. } => signatures
            .get(symbol.as_str())
            .cloned()
            .unwrap_or(Elision::Everything),
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
                | Builtin::ArenaArrayWithCapacity { .. }
                | Builtin::ArenaMapWithCapacity { .. }
                | Builtin::ArenaAlloc { .. }
                | Builtin::ArenaAllocUninit { .. }
                | Builtin::ArenaAllocArrayZeroed { .. }
                | Builtin::ArenaAllocArrayDefault { .. }
                | Builtin::FixedArenaAlloc { .. }
                | Builtin::ScopedArenaAlloc { .. }
                | Builtin::ArenaScope { .. }
                | Builtin::SpanSplitAt { .. }
                | Builtin::Slice { .. }
                | Builtin::StrAsBytes
                | Builtin::SpanReborrow
                | Builtin::SpanSharedReborrow
                | Builtin::SpanChunksNew { .. }
                | Builtin::SpanIterNext { .. }
                | Builtin::SpanChunksNext { .. }
                | Builtin::SpanWindowsNew { .. }
                | Builtin::SpanWindowsNext { .. },
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
        FuncRef::DynBoxNew { .. } => Elision::Nothing,
        // A virtual method has no direct symbol at this stage. Preserve the
        // conservative call contract until a class-specific summary exists.
        FuncRef::Virtual { .. } => Elision::Everything,
        // A dynamic interface method likewise has no statically known
        // implementation summary. Treat it as fully opaque for provenance and
        // borrow checking until interface-object summaries are available.
        FuncRef::Interface { .. } => Elision::Everything,
        // `[LT-7]` — a call through a function value borrows what `[LT-1]`
        // gives a function declared with the callable type's own parameters:
        // its source parameters (rule 3; a callable type has no receiver and
        // no `@borrows`, and a function reaching further is not such a
        // value). A callee of no `fn` type is still opaque.
        FuncRef::Indirect { sources: Some(sources), .. } => Elision::Named(sources.clone()),
        FuncRef::Indirect { sources: None, .. } => Elision::Everything,
    }
}

fn conservative_contract(elision: Elision, latebound: bool) -> CallRegionContract {
    CallRegionContract {
        access: CallAccessContract::All,
        result: CallResultContract::Legacy(elision),
        latebound,
    }
}

fn contract_for(
    func: &FuncRef,
    summaries: &HashMap<String, CallRegionContract>,
    signatures: &HashMap<String, Elision>,
) -> CallRegionContract {
    if let FuncRef::Direct { symbol, .. } = func
        && let Some(summary) = summaries.get(symbol.as_str())
    {
        let latebound = matches!(func, FuncRef::Direct { latebound: true, .. });
        // Ordinary one-region calls retain `[LT-1]`'s conservative elision:
        // every view-typed parameter may be the returned view. A one-field
        // summary is consumed precisely only at an explicit `@latebound`
        // boundary, where it distinguishes a static-independent callback
        // result from a result derived from one of the invocation views.
        if !latebound
            && matches!(&summary.result, CallResultContract::Fields(result)
                if result.fields.len() == 1
                    && result.fields.iter().any(|field| !field.sources.is_empty()))
        {
            return conservative_contract(legacy_elision(func, signatures), false);
        }
        let mut summary = summary.clone();
        summary.latebound = latebound;
        return summary;
    }
    if let Some(contract) = builtin_contract(func) {
        return contract;
    }
    let latebound = matches!(func, FuncRef::Indirect { latebound: true, .. })
        || matches!(func, FuncRef::Direct { latebound: true, .. });
    conservative_contract(legacy_elision(func, signatures), latebound)
}

/// Compiler-known calls whose multi-field result provenance is part of their
/// existing semantic contract. Both halves of a split borrow the same source
/// view even though `[BRW-5]` gives them disjoint storage identities.
fn builtin_contract(func: &FuncRef) -> Option<CallRegionContract> {
    let FuncRef::Builtin {
        which: Builtin::SpanSplitAt { .. },
        ..
    } = func
    else {
        return None;
    };
    let field = |index| ResultFieldProvenance {
        result_projection: vec![Projection::Field(index)],
        sources: vec![ResultRegionSource::View {
            argument: 0,
            projection: Vec::new(),
        }],
    };
    Some(CallRegionContract {
        access: CallAccessContract::All,
        result: CallResultContract::Fields(ResultProvenanceSummary {
            fields: vec![field(0), field(1)],
        }),
        latebound: false,
    })
}

/// Infer exact direct-function result provenance to a fixpoint. A wrapper may
/// depend on a callee declared later, so one source-order pass is insufficient.
/// Unknown and recursive relations remain absent and are conservatively
/// checked (and, when returned, diagnosed as B14) rather than guessed.
fn infer_callable_summaries(
    bodies: &[Body],
    types: &TypeTable,
    signatures: &HashMap<String, Elision>,
) -> HashMap<String, CallRegionContract> {
    let mut summaries: HashMap<String, CallRegionContract> = HashMap::new();
    for _ in 0..=bodies.len() {
        let contract = |func: &FuncRef| contract_for(func, &summaries, signatures);
        let capture_contracts = closure_capture_contracts(bodies, &summaries);
        let mut next = HashMap::new();
        for body in bodies {
            let capture_paths = capture_borrow_paths(body, &capture_contracts);
            let regions = Regions::infer_with_capture_borrow_paths(
                body,
                types,
                &contract,
                &capture_paths,
            );
            next.insert(
                body.symbol.clone(),
                inferred_callable_summary(body, types, &regions),
            );
        }
        if next == summaries {
            return summaries;
        }
        summaries = next;
    }
    summaries
}

fn inferred_callable_summary(
    body: &Body,
    types: &TypeTable,
    regions: &Regions,
) -> CallRegionContract {
    let mut accesses = Vec::new();
    for (local, _) in body.args() {
        let whole_value_move =
            regions.local_regions(local).len() > 1 && !body.borrowed_params.contains(&local);
        for slot in regions.local_regions(local) {
            let mut operations: Vec<RegionAccessKind> =
                regions.accesses(slot.region).iter().copied().collect();
            if whole_value_move && !operations.contains(&RegionAccessKind::Move) {
                operations.push(RegionAccessKind::Move);
            }
            operations.sort();
            if !operations.is_empty() {
                accesses.push(ParameterFieldAccess {
                    argument: local.0 as usize - 1,
                    projection: slot.projection.clone(),
                    operations,
                });
            }
        }
    }

    CallRegionContract {
        access: CallAccessContract::Fields(accesses),
        result: match inferred_result_summary(body, types, regions) {
            Some(summary) => CallResultContract::Fields(summary),
            None => CallResultContract::Legacy(elision_of(body, types)),
        },
        latebound: false,
    }
}

fn inferred_result_summary(
    body: &Body,
    types: &TypeTable,
    regions: &Regions,
) -> Option<ResultProvenanceSummary> {
    let result_slots = regions.local_regions(ember_mir::RETURN_LOCAL);
    if result_slots.is_empty()
        || result_slots
            .iter()
            .any(|slot| regions.has_imprecise_provenance(slot.region))
        || result_slots.iter().any(|slot| {
            regions
                .origins(slot.region)
                .iter()
                .any(|origin| matches!(origin, Origin::Local(_)))
        })
    {
        return None;
    }

    let mut fields = Vec::with_capacity(result_slots.len());
    for result in result_slots {
        let mut sources = Vec::new();
        for (local, decl) in body.args() {
            let argument = local.0 as usize - 1;
            let parameter_slots = regions.local_regions(local);
            if parameter_slots.is_empty() {
                if is_growing_arena_ty(types, decl.ty)
                    && regions
                        .origins(result.region)
                        .contains(&Origin::Param(local))
                {
                    sources.push(ResultRegionSource::Arena { argument });
                }
                continue;
            }
            for parameter in parameter_slots {
                if regions.reaches_slot(parameter.region, result.region) {
                    sources.push(ResultRegionSource::View {
                        argument,
                        projection: parameter.projection.clone(),
                    });
                }
            }
        }
        fields.push(ResultFieldProvenance {
            result_projection: result.projection.clone(),
            sources,
        });
    }
    Some(ResultProvenanceSummary { fields })
}

/// Find bodies whose result summary is invalid because they transitively
/// depend on a genuinely opaque multi-region result. Diagnostics are emitted
/// at the first opaque boundary; callers of an already-invalid body are not
/// flooded with the same recovery error. A direct recursive cycle with no
/// opaque root is deliberately not included and remains diagnosable.
fn infer_invalid_result_bodies(
    bodies: &[Body],
    types: &TypeTable,
    call_contract: &dyn Fn(&FuncRef) -> CallRegionContract,
    capture_contracts: &HashMap<StructId, CallAccessContract>,
) -> HashSet<String> {
    let known: HashSet<&str> = bodies.iter().map(|body| body.symbol.as_str()).collect();
    let regions: Vec<Regions> = bodies
        .iter()
        .map(|body| {
            let capture_paths = capture_borrow_paths(body, capture_contracts);
            Regions::infer_with_capture_borrow_paths(body, types, call_contract, &capture_paths)
        })
        .collect();
    let mut invalid = HashSet::new();

    for (body, regions) in bodies.iter().zip(&regions) {
        if unresolved_multi_result_calls(body, regions, call_contract).any(|func| {
            !matches!(
                func,
                FuncRef::Direct { symbol, .. } if known.contains(symbol.as_str())
            )
        }) {
            invalid.insert(body.symbol.clone());
        }
    }

    let mut changed = true;
    while changed {
        changed = false;
        for (body, regions) in bodies.iter().zip(&regions) {
            if invalid.contains(&body.symbol) {
                continue;
            }
            if unresolved_multi_result_calls(body, regions, call_contract).any(|func| {
                matches!(
                    func,
                FuncRef::Direct { symbol, .. } if invalid.contains(symbol.as_str())
                )
            }) {
                invalid.insert(body.symbol.clone());
                changed = true;
            }
        }
    }
    invalid
}

fn unresolved_multi_result_calls<'a>(
    body: &'a Body,
    regions: &'a Regions,
    call_contract: &'a dyn Fn(&FuncRef) -> CallRegionContract,
) -> impl Iterator<Item = &'a FuncRef> + 'a {
    body.blocks.iter().filter_map(move |block| {
        let Terminator::Call { func, dest, .. } = &block.terminator else {
            return None;
        };
        (regions.place_regions(dest).len() >= 2
            && matches!(call_contract(func).result, CallResultContract::Legacy(_)))
        .then_some(func)
    })
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
    Elision::Named(source_parameters(body, types))
}

/// `[LT-1]` (ODR-024) — the parameters a returned view may borrow without
/// `@borrows`: the source parameters, fixed by the declared signature (a
/// reference or view, or a borrowed or `mut` parameter whose type is not
/// `Copy`). `fn next_token(mut pos: int, src: str) -> str` borrows only `src`.
fn source_parameters(body: &Body, _types: &TypeTable) -> Vec<usize> {
    body.sources.clone()
}

/// `[BRW-4]` — whether this body is a method: its first parameter is the
/// `self` receiver. MIR keeps parameter names, so a caller recognises a
/// method call by its callee without trusting the mangled symbol.
fn is_method_body(body: &Body) -> bool {
    body.arg_count > 0 && body.local(LocalId(1)).name.as_deref() == Some("self")
}

/// `[LT-1]` rule 1 — a borrowed receiver (`self` or `mut self`) that is
/// itself a borrow. An `owned self` is not one: rule 3 takes it with the rest.
fn receiver_is_a_view(body: &Body, types: &TypeTable) -> bool {
    body.arg_count > 0
        && body.local(LocalId(1)).name.as_deref() == Some("self")
        && body.param_modes.first() != Some(&ParameterMode::Owned)
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
    source_parameters(body, types).into_iter().map(|index| LocalId(index as u32 + 1)).collect()
}

/// `E3062`'s offenders: the view parameters a returned view points into
/// that the result may not borrow.
fn return_region_offenders(
    body: &Body,
    types: &TypeTable,
    regions: &Regions,
    own_slot: &HashSet<LocalId>,
) -> Vec<LocalId> {
    if !types.is_view(body.return_ty()) {
        return Vec::new();
    }
    let allowed = allowed_origins(body, types);
    let mut offenders: Vec<LocalId> = regions
        .local_regions(ember_mir::RETURN_LOCAL)
        .iter()
        .flat_map(|slot| regions.origins(slot.region))
        .filter_map(|origin| match origin {
            // A borrow of a by-value parameter is not an elision question at
            // all: nothing in the caller outlives it. `check_escapes` reports
            // that as `E3060`, the same as a local.
            Origin::Param(local)
                if !allowed.contains(local)
                    && types.is_view(body.local(*local).ty)
                    && !own_slot.contains(local) =>
            {
                Some(*local)
            }
            _ => None,
        })
        .collect();
    offenders.sort();
    offenders.dedup();
    offenders
}

/// `E3062` — the returned view points into a parameter elision did not tie it
/// to (shape B6).
///
/// This is the half of `[LT-1a]` that needed regions. The attribute was
/// checked as a *signature* — that it names parameters, that they are
/// view-typed, that the return is a view — and the body was free to contradict
/// it. `@borrows(a)` on a function that returns `b` compiled, and the caller
/// then went on using `b` while holding a reference into it.
fn check_return_regions(
    body: &Body,
    types: &TypeTable,
    regions: &Regions,
    own_slot: &HashSet<LocalId>,
    sink: &mut Sink,
) {
    if !types.is_view(body.return_ty()) {
        return;
    }
    let return_regions = regions.local_regions(ember_mir::RETURN_LOCAL);
    if return_regions.is_empty() {
        return;
    }
    if return_regions.iter().any(|slot| {
        regions
            .origins(slot.region)
            .iter()
            .any(|origin| matches!(origin, Origin::LateBound { .. }))
    }) {
        sink.emit_classified(
            Diagnostic::error(
                codes::E3062,
                body.span,
                "a `@latebound` callback result contains an invocation-local view region",
            )
            .primary_label("the callback-local view would escape here")
            .help("return an owned value, or keep callback-local views inside the callback invocation")
            .note("a `@latebound` callable receives fresh invocation-local regions that cannot escape its boundary (FN-6b, LT-7, LT-10)"),
        );
    }
    let allowed = allowed_origins(body, types);
    let Some(span) = body
        .blocks
        .iter()
        .find(|b| matches!(b.terminator, Terminator::Return))
        .map(|b| b.terminator_span)
    else {
        return;
    };

    let offenders = return_region_offenders(body, types, regions, own_slot);

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
        } else if receiver_is_a_view(body, types) {
            (
                format!("the returned view points into `{name}` rather than into `self`"),
                format!("write `@borrows({name})` above the declaration, which overrides rule 1"),
            )
        } else {
            // `[LT-1]` (ODR-024) — a `mut` parameter of a `Copy` type is not
            // a source: its value can be returned instead.
            (
                format!("the returned view points into `{name}`, which the result does not borrow"),
                format!("write `@borrows({name})` above the declaration, or return the value instead of a view of it"),
            )
        };
        // `[LT-1a]` — `@borrows` may name only a source parameter or a `mut`
        // one; a borrowed `Copy` parameter (a generic `x: T`, a `Copy` struct
        // holding a `Cell`) gets a help that compiles when applied.
        let index = (local.0 as usize).wrapping_sub(1);
        let nameable = body.param_modes.get(index) == Some(&ParameterMode::Mut) || body.sources.contains(&index);
        let help = if body.is_lambda {
            // A lambda carries no `@borrows` (C4).
            "return a value, or a view of one of the lambda's parameters or captures: a lambda \
             cannot carry `@borrows`"
                .to_string()
        } else if nameable {
            help
        } else if match types.kind(decl.ty) {
            TyKind::Fn { .. } => true,
            // A callable parameter instantiated with a lambda has the lambda's
            // environment type, named `closure{n}_env` by typeck's
            // `push_closure`. ponytail: matched by that name, since StructDef
            // has no closure flag; add one if another pass needs to know.
            TyKind::Struct(id) => {
                let name = types.struct_def(*id).name;
                name.as_str().starts_with("closure") && name.as_str().ends_with("_env")
            }
            _ => false,
        } {
            // `[CLO-3]` — a callable parameter is never a source.
            "return an owned value: a callable parameter is not one a result can borrow (LT-1)".to_string()
        } else {
            format!(
                "return the value instead of a view of it, or take `{name}` as a `ref` parameter \
                 (a caller passing a place keeps the same call, TYP-5 rule 7); a borrowed `Copy` \
                 parameter is not one a result can borrow (LT-1)"
            )
        };
        sink.emit_classified(
            Diagnostic::error(codes::E3062, span, message)
                .primary_label("returned here")
                .secondary(
                    decl.span,
                    format!("`{name}` is the parameter it points into"),
                )
                .help(help)
                .note("`@borrows` is what ties a return to a parameter elision would not (LT-1a)"),
        );
    }
}

/// `[LT-22]`, shape B14 — every call producing a multi-region result must have
/// an exact field-to-input provenance relation. An opaque/recursive result
/// edge is rejected at that boundary rather than widened to the legacy
/// intersection or treated as static.
fn check_multi_result_summary(
    body: &Body,
    types: &TypeTable,
    regions: &Regions,
    call_contract: &dyn Fn(&FuncRef) -> CallRegionContract,
    invalid_result_bodies: &HashSet<String>,
    sink: &mut Sink,
) {
    for block in &body.blocks {
        let Terminator::Call { func, dest, .. } = &block.terminator else {
            continue;
        };
        if regions.place_regions(dest).len() < 2
            || matches!(call_contract(func).result, CallResultContract::Fields(_))
            || matches!(
                func,
                FuncRef::Direct { symbol, .. } if invalid_result_bodies.contains(symbol.as_str())
            )
        {
            continue;
        }
        let result = types.display(place_ty(body, types, dest));
        sink.emit_classified(
            Diagnostic::error(
                codes::E3065,
                block.terminator_span,
                format!(
                    "the returned `{result}` has field provenance that cannot be inferred soundly"
                ),
            )
            .primary_label("multi-region result returned here")
            .secondary(
                body.span,
                "this function must expose one source relation per borrowed result field",
            )
            .help(
                "return an owned value, return the views separately, or use an applicable existing `@borrows` contract",
            )
            .note(
                "the compiler will not force an intersection or invent a `static` region (LT-22)",
            ),
        );
    }
}

/// `[TYP-15]` — an `Array`'s elements live on the heap, which has no bounding
/// region, so a view may enter one only when every region it carries is
/// `static`: `names = ["ann", "bob"]` is an `Array[str]` of static strings, and
/// `[xs[2..]]` is `E3063`. Checked where an element enters: a list literal,
/// `push`, `insert` and `a[i] = v` (D-199). A `mut` parameter or `mem.replace`
/// aimed at an element is D-198's half, which needs heap-element views to be
/// `static` in the region model itself.
fn check_array_storage_regions(body: &Body, types: &TypeTable, regions: &Regions, sink: &mut Sink) {
    let operand_ty = |operand: &Operand| match operand {
        Operand::Copy(place) | Operand::Move(place) => Some(place_ty(body, types, place)),
        Operand::Const(_) => None,
    };
    // A view read out of an `Array` whose elements are exactly its type is
    // itself one of those elements, and every element was checked `static`
    // where it entered: `[n for n in names]` copies static strings.
    let element_read = |operand: &Operand, value: Ty, point: Point| {
        regions.operand_origins_at(operand, point).is_some_and(|origins| {
            !origins.is_empty()
                && origins.iter().all(|origin| {
                    let (Origin::Local(local) | Origin::Param(local)) = origin else { return false };
                    let mut container = body.local(*local).ty;
                    if let TyKind::Ref { inner, .. } = *types.kind(container) {
                        container = inner;
                    }
                    matches!(*types.kind(container), TyKind::Vec { elem } if elem == value)
                })
        })
    };
    let is_static = |operand: &Operand, point: Point| {
        let Some(ty) = operand_ty(operand) else { return true };
        let value = match *types.kind(ty) {
            TyKind::Array { elem, .. } if types.is_view(elem) => elem,
            _ if types.is_view(ty) => ty,
            _ => return true,
        };
        regions.is_static_operand_at(operand, point) || element_read(operand, value, point)
    };
    let mut report = |span: Span, elem: Ty| {
        let shown = types.display(elem);
        sink.emit_classified(
            Diagnostic::error(
                codes::E3063,
                span,
                format!("`{shown}` is a view, so it may not be stored in an `Array` unless it is `static`"),
            )
            .primary_label("stored here")
            .help(concat!(
                "store an owned copy — `String` for `str`, `Array[T]` for `Span[T]` — ",
                "and note that costs one allocation per element; or store a `u32` index ",
                "or a `Handle[T]` and name the container it indexes"
            ))
            .note(concat!(
                "an `Array`'s elements have no bounding region, so only a view with the ",
                "`static` region may be stored in them, such as a string literal (TYP-15, LT-3)"
            )),
        );
    };
    for (block_index, block) in body.blocks.iter().enumerate() {
        for (index, stmt) in block.stmts.iter().enumerate() {
            let StmtKind::Assign { place, rvalue } = &stmt.kind else { continue };
            let Some((last, prefix)) = place.projection.split_last() else { continue };
            if !matches!(last, Projection::Index(_) | Projection::ConstIndex(_)) {
                continue;
            }
            let base = Place { local: place.local, projection: prefix.to_vec() };
            let TyKind::Vec { elem } = *types.kind(place_ty(body, types, &base)) else { continue };
            if !types.is_view(elem) {
                continue;
            }
            let point = Point { block: block_index, index };
            let escapes = match rvalue {
                Rvalue::Use(operand) => !is_static(operand, point),
                Rvalue::Aggregate { operands, .. } => operands.iter().any(|operand| !is_static(operand, point)),
                Rvalue::Ref { .. } => true,
                _ => false,
            };
            if escapes {
                report(stmt.span, elem);
            }
        }
        // `a[i] = f(…)` writes the call's result straight into the element;
        // the result can only be as static as the views it was given.
        if let Terminator::Call { dest, args, .. } = &block.terminator
            && let Some((Projection::Index(_) | Projection::ConstIndex(_), prefix)) = dest.projection.split_last()
            && let TyKind::Vec { elem } =
                *types.kind(place_ty(body, types, &Place { local: dest.local, projection: prefix.to_vec() }))
            && types.is_view(elem)
        {
            let point = Point { block: block_index, index: block.stmts.len() };
            if args.iter().any(|arg| !is_static(arg, point)) {
                report(block.terminator_span, elem);
            }
        }
        let Terminator::Call { func: FuncRef::Builtin { which, .. }, args, .. } = &block.terminator else {
            continue;
        };
        let value = match which {
            Builtin::ArrayFromLiteral => args.first(),
            Builtin::ArrayPush | Builtin::ArrayInsert => args.get(1),
            _ => None,
        };
        let Some(value) = value else { continue };
        let Some(ty) = operand_ty(value) else { continue };
        let elem = match *types.kind(ty) {
            TyKind::Array { elem, .. } if matches!(which, Builtin::ArrayFromLiteral) => elem,
            _ => ty,
        };
        let point = Point { block: block_index, index: block.stmts.len() };
        if types.is_view(elem) && !is_static(value, point) {
            report(block.terminator_span, elem);
        }
    }
}

/// `[TYP-15]`, `[LT-3]` — an unbounded Box or Shared owner has no bounding region, so a view may enter it
/// only when every carried region is static. This check belongs after region
/// inference: spelling the same static view through a local or a zero-input
/// function must not change whether the program is accepted.
fn check_box_storage_regions(body: &Body, types: &TypeTable, regions: &Regions, sink: &mut Sink) {
    for (block_index, block) in body.blocks.iter().enumerate() {
        let Terminator::Call {
            func:
                FuncRef::Builtin {
                    which:
                        builtin @ (Builtin::BoxNew { elem, .. } | Builtin::SharedNew { elem, .. }),
                    ..
                },
            args,
            ..
        } = &block.terminator
        else {
            continue;
        };
        let point = Point {
            block: block_index,
            index: block.stmts.len(),
        };
        if !types.is_view(*elem)
            || args
                .first()
                .is_some_and(|value| regions.is_static_operand_at(value, point))
        {
            continue;
        }
        let shown = types.display(*elem);
        let owner = match builtin {
            Builtin::BoxNew { .. } => "Box",
            Builtin::SharedNew { .. } => "Shared",
            _ => unreachable!("only BoxNew and SharedNew reach this check"),
        };
        sink.emit_classified(
            Diagnostic::error(
                codes::E3063,
                block.terminator_span,
                format!("`{shown}` is a view, so it may not be stored in a {owner}'s contents"),
            )
            .primary_label("stored here")
            .help(concat!(
                "store an owned copy — `String` for `str`, `Array[T]` for `Span[T]` — ",
                "and note that costs one allocation per element; or store a `u32` index ",
                "or a `Handle[T]` and name the container it indexes"
            ))
            .note(concat!(
                "this place has no bounding region, so only a view with the `static` ",
                "region may be stored in it (TYP-15, LT-3)"
            )),
        );
    }
}

/// `[CLO-4]`, `[TYP-15]` — an ordinary capturing closure contains reference
/// fields. Passing it by `owned` mode gives the callee ownership of a value it
/// may retain beyond this call, so the same unbounded-storage boundary applies
/// at the call itself. This is deliberately narrower than rejecting all
/// owned view arguments: it applies only to compiler-generated environments
/// that capture by reference, and consults actual region provenance so a
/// future static environment remains valid.
fn check_owned_closure_argument_regions(
    body: &Body,
    types: &TypeTable,
    regions: &Regions,
    borrowing_closure_environments: &HashSet<StructId>,
    direct_param_modes: &HashMap<String, Vec<ParameterMode>>,
    sink: &mut Sink,
) {
    for (block_index, block) in body.blocks.iter().enumerate() {
        let Terminator::Call { func: FuncRef::Direct { symbol, .. }, args, .. } = &block.terminator
        else {
            continue;
        };
        let Some(modes) = direct_param_modes.get(symbol) else {
            // An imported callable needs its parameter modes in its interface
            // artifact before this check can make a storage claim. The
            // separate-compilation boundary remains conservative elsewhere.
            continue;
        };
        let point = Point {
            block: block_index,
            index: block.stmts.len(),
        };
        for (argument, mode) in args.iter().zip(modes) {
            if *mode != ParameterMode::Owned {
                continue;
            }
            let (Operand::Copy(place) | Operand::Move(place)) = argument else {
                continue;
            };
            let TyKind::Struct(environment) = *types.kind(place_ty(body, types, place)) else {
                continue;
            };
            if !borrowing_closure_environments.contains(&environment)
                || regions.is_static_operand_at(argument, point)
            {
                continue;
            }
            sink.emit_classified(
                Diagnostic::error(
                    codes::E3063,
                    block.terminator_span,
                    "a closure that captures by reference cannot be passed to an `owned` callable parameter",
                )
                .primary_label("moved across an ownership boundary here")
                .help(
                    "use `owned fn` to capture owned values, or take the callable as a borrowed or `mut` parameter and invoke it before its captures end",
                )
                .note(
                    "an ordinary closure is a view over its captures; an `owned` callable parameter may retain that view (CLO-4, TYP-15)",
                ),
            );
        }
    }
}

/// `[LT-42]` with `[TYP-15]` — an `owned fn` environment owns its captures and
/// can therefore escape the frame that constructed it. A captured view is
/// consequently valid only when every carried region is static. This is
/// checked where lowering constructs the compiler-generated environment, not
/// by treating the environment name or C representation as source semantics.
fn check_owned_closure_capture_regions(
    body: &Body,
    types: &TypeTable,
    regions: &Regions,
    owned_closure_environments: &HashSet<StructId>,
    sink: &mut Sink,
) {
    if owned_closure_environments.is_empty() {
        return;
    }
    for (block_index, block) in body.blocks.iter().enumerate() {
        for (index, statement) in block.stmts.iter().enumerate() {
            let StmtKind::Assign {
                rvalue:
                    Rvalue::Aggregate {
                        kind: AggregateKind::Struct(environment),
                        operands,
                    },
                ..
            } = &statement.kind
            else {
                continue;
            };
            if !owned_closure_environments.contains(environment) {
                continue;
            }
            let point = Point { block: block_index, index };
            for (field, operand) in types.struct_def(*environment).fields.iter().zip(operands) {
                if !types.is_view(field.ty) || regions.is_static_operand_at(operand, point) {
                    continue;
                }
                let shown = types.display(field.ty);
                sink.emit_classified(
                    Diagnostic::error(
                        codes::E3063,
                        statement.span,
                        format!(
                            "`owned fn` captures `{shown}` by value, but that view is not static"
                        ),
                    )
                    .primary_label("captured into an escaping closure here")
                    .help(
                        "capture an owned value instead, or use a non-`owned` closure whose lifetime is bounded by the borrowed source",
                    )
                    .note(
                        "an `owned fn` can escape its defining frame, so every view region it captures must be static (LT-42, TYP-15)",
                    ),
                );
            }
        }
    }
}

pub fn check(body: &Body, types: &TypeTable, sink: &mut Sink) {
    let is_method = |func: &FuncRef| match func {
        FuncRef::Direct { symbol, .. } => {
            symbol.as_str() == body.symbol.as_str() && is_method_body(body)
        }
        _ => false,
    };
    check_body(
        body,
        types,
        &|_| conservative_contract(Elision::Everything, false),
        &HashMap::new(),
        &HashSet::new(),
        &HashSet::new(),
        &HashMap::new(),
        &HashSet::new(),
        &is_method,
        sink,
    );
}

fn check_body(
    body: &Body,
    types: &TypeTable,
    call_contract: &dyn Fn(&FuncRef) -> CallRegionContract,
    capture_paths: &HashMap<Point, Vec<Vec<Projection>>>,
    owned_closure_environments: &HashSet<StructId>,
    borrowing_closure_environments: &HashSet<StructId>,
    direct_param_modes: &HashMap<String, Vec<ParameterMode>>,
    invalid_result_bodies: &HashSet<String>,
    is_method: &dyn Fn(&FuncRef) -> bool,
    sink: &mut Sink,
) {
    let regions = Regions::infer_with_capture_borrow_paths(
        body,
        types,
        call_contract,
        capture_paths,
    );
    check_box_storage_regions(body, types, &regions, sink);
    check_array_storage_regions(body, types, &regions, sink);
    check_owned_closure_argument_regions(
        body,
        types,
        &regions,
        borrowing_closure_environments,
        direct_param_modes,
        sink,
    );
    check_owned_closure_capture_regions(body, types, &regions, owned_closure_environments, sink);
    // A function that hands back a parameter has no loan of its own, and
    // `[LT-1a]` is about exactly that function. A reference to a view
    // parameter's own slot is not a question of which parameter the result
    // borrows: it is `E3060` (`[BRW-8]`), and `check_escapes` reports it.
    let loans = collect_loans(body, types, &regions, call_contract);
    let own_slot = own_slot_returns(body, types, &loans, &regions);
    check_return_regions(body, types, &regions, &own_slot, sink);
    check_multi_result_summary(
        body,
        types,
        &regions,
        call_contract,
        invalid_result_bodies,
        sink,
    );
    if loans.is_empty() {
        return;
    }
    let reads = collect_reads(body);
    let mut reported = HashSet::new();

    for (block_index, block) in body.blocks.iter().enumerate() {
        for (index, stmt) in block.stmts.iter().enumerate() {
            let point = Point {
                block: block_index,
                index,
            };
            let mut accesses = Vec::new();
            match &stmt.kind {
                StmtKind::Assign { place, rvalue } => {
                    // A borrow is not a conflicting read of its own operand.
                    if let Rvalue::Ref {
                        place: borrowed,
                        mutable,
                    } = rvalue
                    {
                        if let Some(paths) = regions.capture_borrow_paths(point) {
                            for path in paths {
                                accesses.push((
                                    project_place(borrowed, path),
                                    Access::Borrow { mutable: *mutable },
                                ));
                            }
                        } else {
                            accesses.push((borrowed.clone(), Access::Borrow { mutable: *mutable }));
                        }
                    } else {
                        rvalue_reads(rvalue, &mut accesses);
                    }
                    accesses.push((place.clone(), Access::Write));
                }
                StmtKind::CheckedBinaryOp {
                    dest,
                    overflow,
                    lhs,
                    rhs,
                    ..
                } => {
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
                    //
                    // D-190 — the same holds for any storage a loan still
                    // points into at the `return`: that loan escapes, and
                    // `check_escapes` reports it (B7, `E3060`). Reporting the
                    // drop as a write too (`E3021`) is two errors for one
                    // mistake (`[DIA-14]`).
                    let returning = matches!(block.terminator, Terminator::Return);
                    let return_point = Point { block: block_index, index: block.stmts.len() };
                    let escapes_at_return = returning
                        && (is_arena_ty(types, place_ty(body, types, place))
                            || in_scope(&loans, &regions, return_point).iter().any(|loan| {
                                loan.capability.must_not_outlive_storage()
                                    && loan
                                        .capability
                                        .source_place()
                                        .is_some_and(|source| source.local == place.local)
                            }));
                    if !escapes_at_return {
                        // A drop ends the place's own storage, so it conflicts
                        // only with loans that depend on that storage; a view
                        // handed out by a span built-in points at the span's
                        // target (D-217), which the drop does not touch.
                        let depending: Vec<Loan> = loans
                            .iter()
                            .filter(|loan| loan.capability.must_not_outlive_storage())
                            .cloned()
                            .collect();
                        check_point(
                            body,
                            types,
                            &depending,
                            &regions,
                            &reads,
                            point,
                            &[(place.clone(), Access::Write)],
                            stmt.span,
                            is_method,
                            sink,
                            &mut reported,
                        );
                    }
                }
                // Dynamic class access intervals are checked by the runtime;
                // they do not create static loans for the ordinary borrow
                // checker to compare here.
                StmtKind::BeginAccess { .. }
                | StmtKind::BeginAccessTransfer { .. }
                | StmtKind::EndAccess { .. }
                | StmtKind::EndAccessTransfer { .. }
                | StmtKind::StorageLive(_)
                | StmtKind::StorageDead(_)
                | StmtKind::Nop => {}
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

        let point = Point {
            block: block_index,
            index: block.stmts.len(),
        };
        let mut accesses = Vec::new();
        match &block.terminator {
            Terminator::SwitchInt { discr, .. } => operand_read(discr, &mut accesses),
            Terminator::Call { args, dest, .. } => {
                check_call_activation(
                    body,
                    types,
                    &loans,
                    &regions,
                    &reads,
                    point,
                    args,
                    block.terminator_span,
                    is_method,
                    sink,
                    &mut reported,
                );
                accesses.push((dest.clone(), Access::Write));
            }
            Terminator::Assert { cond, msg, .. } => {
                operand_read(cond, &mut accesses);
                if let ember_mir::AssertKind::RefCellBorrow { file, line } = msg {
                    operand_read(file, &mut accesses);
                    operand_read(line, &mut accesses);
                }
                if let ember_mir::AssertKind::Panic { message } = msg {
                    operand_read(message, &mut accesses);
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
        if let Terminator::Call { func, .. } = &block.terminator
            && !builtin_cannot_reach_a_cell(func)
        {
            check_refcell_call(
                body,
                types,
                &loans,
                &regions,
                point,
                block.terminator_span,
                sink,
            );
        }

        // §4.7 step 6 — a loan still live where the borrowed place's storage
        // ends. Returning is the case that matters: the reference leaves the
        // frame while what it points at does not.
        if matches!(block.terminator, Terminator::Return) {
            check_escapes(
                body,
                types,
                &loans,
                &regions,
                point,
                block.terminator_span,
                sink,
            );
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
        // `[IMP-7]` — enforce only the storage-survival obligation carried by
        // the canonical capability. Current reference producers all include
        // it; future raw/handle/observing capabilities may have different
        // explicitly declared escape constraints.
        if !loan.capability.must_not_outlive_storage() {
            continue;
        }
        let place = loan
            .capability
            .source_place()
            .expect("a loan has source storage");
        let root = body.local(place.local);
        let is_parameter = matches!(
            loan.capability.provenance_root,
            ProvenanceRoot::Param(_)
        );
        // `[IMP-7]` — allocation-return loans carry a canonical concrete
        // storage owner. Keep the source-type fallback for ordinary explicit
        // borrows of an Arena value, whose storage identity is still a place
        // root rather than an allocation site.
        let arena_owner = loan
            .capability
            .storage_identity
            .arena_owner()
            .or_else(|| is_arena_ty(types, root.ty).then_some(place.local));
        // D-217 — a loan that reaches its memory through a view, a
        // parameter's or a local's, points where the view does: the view's own loans keep what
        // it points to alive, and report it if that is this frame's (a local
        // array's span). A view parameter's own slot is this frame's
        // (`return ref x` for `x: str`), and so is an `Array` or `Box` it owns.
        let through_a_view = types.is_view(root.ty) && through_indirection(body, types, place);
        if through_a_view
            || (is_parameter
                && arena_owner.is_some_and(|owner| is_named_arena_origin(body, owner, types)))
        {
            continue;
        }
        let name = place_name(body, types, place);
        // The label is about the *owner*, which for `self.n` is `self`.
        let owner = place_name(body, types, &Place::local(place.local));
        // `[BRW-8]` (ODR-024) — a parameter a result cannot borrow is either
        // `owned` (the callee's own) or a `Copy` value passed as a copy.
        let owned_parameter = is_parameter
            && body.param_modes.get((place.local.0 as usize).wrapping_sub(1)) == Some(&ParameterMode::Owned);
        let own_view_slot = is_parameter && types.is_view(root.ty);
        let storage = if own_view_slot {
            format!("`{owner}` is a view passed as a copy; a reference to the view itself points into this frame")
        } else if owned_parameter {
            format!("`{owner}` is `owned`, so it is the callee's own and its storage ends with the frame")
        } else if is_parameter {
            format!("`{owner}` is passed as a copy, so the copy's storage ends with the frame")
        } else {
            format!("`{owner}` is a local, so its storage ends with the frame")
        };
        let returns_a_ref_to_the_view = matches!(
            types.kind(body.return_ty()),
            TyKind::Ref { inner, .. } if *inner == root.ty
        );
        let repair = if own_view_slot && returns_a_ref_to_the_view {
            let view = types.display(root.ty);
            format!("return `{view}` instead of `ref {view}`: `{owner}` already borrows the caller's data")
        } else if own_view_slot {
            format!("return a view of what `{owner}` points to, or an owned value")
        } else if owned_parameter {
            format!("borrow `{owner}` instead of taking it `owned`, or return an owned value")
        } else if is_parameter {
            format!("take `{owner}` as a view (`Span[T]`, `str` or `ref T`), or return an owned value")
        } else {
            "return an owned value, take the destination as a `mut` parameter, or borrow something the caller owns"
                .to_string()
        };
        let is_arena = arena_owner.is_some();
        if is_arena {
            let owner_local = arena_owner.expect("arena diagnostic has an owner");
            let owner_name = place_name(body, types, &Place::local(owner_local));
            let (origin_label, help, note) = if is_parameter {
                (
                    format!(
                        "`{owner_name}` is a parameter, but the signature does not tie the return to it"
                    ),
                    format!(
                        "write `@borrows({owner_name})` above the wrapper, or return an owned value"
                    ),
                    "an Arena parameter is a return-provenance source only when `@borrows` names it (LT-4a)",
                )
            } else {
                (
                    format!("`{owner_name}` is local to this frame"),
                    "move the `Arena` to an outer scope, or copy the value out before it is reset"
                        .to_string(),
                    "Arena allocation views carry the region of the arena borrow (LT-4, ARN-1)",
                )
            };
            sink.emit_classified(
                Diagnostic::error(
                    codes::E3061,
                    span,
                    format!("arena allocation cannot outlive `{owner_name}`"),
                )
                .primary_label("this allocation view escapes the arena's region")
                .secondary(
                    loan.span,
                    format!("`{owner_name}` is borrowed for the allocation here"),
                )
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
                .help(repair)
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

/// `[CELL-7]` (F-187) — `L3011` fires only for calls that can reach the cell,
/// never across one that provably cannot. These compiler-known operations
/// run no Ember code: they print, format, read or build text and views,
/// compare, or grow an `Array` without dropping anything. An operation that
/// may drop a value (whose `drop` could reach the cell) or call a function
/// is not listed.
fn builtin_cannot_reach_a_cell(func: &FuncRef) -> bool {
    let FuncRef::Builtin { which, .. } = func else { return false };
    matches!(
        which,
        Builtin::Print
            | Builtin::EPrint
            | Builtin::Println
            | Builtin::EPrintln
            | Builtin::Format
            | Builtin::FormatWith(_)
            | Builtin::StringNew
            | Builtin::StringPush
            | Builtin::StringLen
            | Builtin::StringAsStr
            | Builtin::StrCharCount
            | Builtin::StrStartsWith
            | Builtin::StrEndsWith
            | Builtin::StrFind { .. }
            | Builtin::StrCount
            | Builtin::StrReplace
            | Builtin::StrRepeat
            | Builtin::StrTrimStart
            | Builtin::StrTrimEnd
            | Builtin::StrSliceOk
            | Builtin::StrToUpper
            | Builtin::ParseStatus { .. }
            | Builtin::ParseValue { .. }
            | Builtin::StrToLower
            | Builtin::StrCharAt
            | Builtin::CharUtf8Len
            | Builtin::StrContains
            | Builtin::StrContainsChar
            | Builtin::StrIsCharBoundary
            | Builtin::StrAsBytes
            | Builtin::Slice { .. }
            | Builtin::SpanLen
            | Builtin::SpanGet
            | Builtin::SpanFrom { .. }
            | Builtin::ArrayLen
            | Builtin::ArrayNew
            | Builtin::ArrayPush
            | Builtin::ValueCompare { .. }
            | Builtin::TotalLess
            | Builtin::FloatAbs
            | Builtin::FloatPow
            | Builtin::FloatLib(_)
            | Builtin::IntBits(_)
            | Builtin::RangeCount
            | Builtin::RangeNth
            | Builtin::SizeOf
            | Builtin::AlignOf
    )
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
        .filter(|loan| {
            // `collect_loans` is the producer for this canonical fact. Do not
            // re-derive the loan's guard identity from its source-place shape
            // after the fact; that would leave the consumer coupled to one
            // lowering representation instead of the verified capability.
            loan.capability.reference_kind == ReferenceKind::RuntimeGuard
        })
        .collect();
    live.sort_by_key(|loan| (loan.created_at.block, loan.created_at.index));
    let Some(loan) = live.first() else {
        return;
    };
    // Name the cell, not its private `value` field: the loan is of field 0,
    // which no source ever writes. Strip one trailing `Field(0)` where the
    // parent is the cell for the display; the loan itself is unchanged.
    let mut display = loan
        .capability
        .source_place()
        .expect("a guard loan has source storage")
        .clone();
    if let Some(Projection::Field(0)) = display.projection.last() {
        let mut parent = display.clone();
        parent.projection.pop();
        if is_refcell_ty(types, place_ty(body, types, &parent)) {
            display = parent;
        }
    }
    let cell = place_name(body, types, &display);
    sink.emit(
        Diagnostic::lint(
            codes::L3011,
            span,
            format!("a `RefCell` guard for `{cell}` is live across this call"),
        )
        .primary_label("call happens here")
        .secondary(loan.span, "guard borrowed here")
        .help("drop the guard before the call, or put the call in a block of its own")
        .note(
            "a guard live across a call that re-enters the same cell panics at run time [CELL-7]",
        ),
    );
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
    call_contract: &dyn Fn(&FuncRef) -> CallRegionContract,
) -> Vec<Loan> {
    let mut loans = Vec::new();
    for (block_index, block) in body.blocks.iter().enumerate() {
        for (index, stmt) in block.stmts.iter().enumerate() {
            let StmtKind::Assign { place, rvalue } = &stmt.kind else {
                continue;
            };
            let Rvalue::Ref {
                place: borrowed,
                mutable,
            } = rvalue
            else {
                continue;
            };
            let created_at = Point {
                block: block_index,
                index,
            };
            let Some(region) = regions.loan_region(created_at) else {
                continue;
            };
            let permission = if *mutable {
                AccessPermission::Mut
            } else {
                AccessPermission::Shared
            };
            let reference_kind = if is_guard_loan(body, types, borrowed) {
                ReferenceKind::RuntimeGuard
            } else {
                ReferenceKind::Reference
            };
            let paths: &[Vec<Projection>] = regions
                .capture_borrow_paths(created_at)
                .unwrap_or(&[]);
            let borrowed_places: Vec<Place> = if paths.is_empty() {
                vec![borrowed.clone()]
            } else {
                paths.iter().map(|path| project_place(borrowed, path)).collect()
            };
            // D-217 — a loan of a span taken only to hand it to a built-in
            // (`&mut span` fed to `reborrow` or `split_at`) gives out what the
            // span points to, not its slot, so the slot's storage obligation
            // does not apply. The loan still conflicts with every other use.
            for borrowed in borrowed_places {
                let feeds_reborrow = borrower_feeds_span_builtin(body, types, place.local, &borrowed);
                let storage_root = borrowed.local;
                let mut capability = BorrowCapability::statically_checked_reference(
                    place_ty(body, types, &borrowed),
                    provenance_root(body, storage_root),
                    borrowed,
                    StorageIdentity::PlaceRoot(storage_root),
                    region,
                    permission,
                    reference_kind,
                );
                if feeds_reborrow {
                    capability.escape_constraints.remove(&EscapeConstraint::MustNotOutliveStorage);
                }
                loans.push(Loan {
                    capability,
                    borrower: place.local,
                    created_at,
                    span: stmt.span,
                    reserved_at: reservation_window(body, place.local, created_at),
                    arena_scope: borrower_feeds_arena_scope(body, place.local),
                });
            }
        }

        let Terminator::Call {
            func, args, dest, ..
        } = &block.terminator
        else {
            continue;
        };
        let Some(region) = regions.local_region(dest.local) else {
            continue;
        };
        let tied = call_contract(func);
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
            let created_at = Point {
                block: block_index,
                index: block.stmts.len(),
            };
            let capability = BorrowCapability::statically_checked_reference(
                place_ty(body, types, borrowed),
                provenance_root(body, borrowed.local),
                borrowed.clone(),
                StorageIdentity::ArenaAllocation {
                    arena: borrowed.local,
                    site: created_at,
                },
                region,
                AccessPermission::Shared,
                ReferenceKind::Reference,
            );
            loans.push(Loan {
                capability,
                borrower: dest.local,
                created_at,
                span: block.terminator_span,
                reserved_at: HashSet::new(),
                arena_scope: false,
            });
        }
    }
    loans
}

fn provenance_root(body: &Body, local: LocalId) -> ProvenanceRoot {
    match body.local(local).kind {
        LocalKind::Arg => ProvenanceRoot::Param(local),
        _ => ProvenanceRoot::Local(local),
    }
}

/// Whether a compiler temporary holds a borrow of a `Span` or `str` only to
/// hand it, as the receiver, to a compiler built-in (`reborrow`, `split_at`,
/// `iter_mut`'s reborrow, `chunks_mut`, …). A built-in never returns a
/// reference to a view's own slot: whatever it hands back points where the
/// view does (D-217).
fn borrower_feeds_span_builtin(body: &Body, types: &TypeTable, borrower: LocalId, borrowed: &Place) -> bool {
    body.local(borrower).kind == LocalKind::Temp
        && matches!(types.kind(place_ty(body, types, borrowed)), TyKind::Span { .. } | TyKind::Str)
        && body.blocks.iter().any(|block| {
            matches!(
                &block.terminator,
                Terminator::Call { func: FuncRef::Builtin { .. }, args, .. }
                    if matches!(args.first(), Some(Operand::Copy(place) | Operand::Move(place))
                        if place.local == borrower && place.projection.is_empty())
            )
        })
}

fn borrower_feeds_arena_scope(body: &Body, borrower: LocalId) -> bool {
    body.blocks.iter().any(|block| {
        let Terminator::Call {
            func:
                FuncRef::Builtin {
                    which: Builtin::ArenaScope { .. },
                    ..
                },
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
        let Some(block) = body.blocks.get(block_index) else {
            return HashSet::new();
        };

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
            window.insert(Point {
                block: block_index,
                index,
            });
        }

        let terminator_point = Point {
            block: block_index,
            index: block.stmts.len(),
        };
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
    let StmtKind::Assign {
        place: borrower,
        rvalue: Rvalue::Ref {
            place: borrowed,
            mutable: true,
        },
    } = &stmt.kind
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
            Terminator::Call {
                func, args, next, ..
            } => {
                let receiver = args.first().and_then(|arg| match arg {
                    Operand::Copy(p) | Operand::Move(p) => Some(p.local),
                    Operand::Const(_) => None,
                });
                if receiver == Some(borrower) {
                    // The temporary is this call's receiver: B8 exactly when
                    // the callee is a method.
                    return if is_method(func) {
                        Some(borrowed)
                    } else {
                        None
                    };
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
    loans
        .iter()
        .filter(|loan| regions.contains(loan.capability.validity_interval.0, point))
        .collect()
}

/// `[BRW-3]` — a call's arguments are evaluated before the call starts, and a
/// `mut` argument's loan is activated when it starts. So the arguments are
/// read with the loans this call activates still reserved (`f(v, v[0])` with
/// `x: int` copies `v[0]` first), and then each activated loan is a new
/// mutable borrow against the shared loans taken while it was reserved: the
/// call's other arguments (`f(v, v[0])` with `x: ref int` is `E3021`, as the
/// other order is). Loans older than the reservation were checked when it
/// began, and a second `mut` argument is `E3022` there, so neither is
/// reported twice.
#[allow(clippy::too_many_arguments)]
fn check_call_activation(
    body: &Body,
    types: &TypeTable,
    loans: &[Loan],
    regions: &Regions,
    reads: &HashMap<LocalId, Vec<Span>>,
    point: Point,
    args: &[Operand],
    span: Span,
    is_method: &dyn Fn(&FuncRef) -> bool,
    sink: &mut Sink,
    reported: &mut HashSet<(usize, usize)>,
) {
    let passes = |local: LocalId| {
        args.iter().any(|arg| matches!(arg, Operand::Copy(p) | Operand::Move(p) if p.local == local))
    };
    let activated: Vec<usize> = loans
        .iter()
        .enumerate()
        .filter(|(_, loan)| loan.capability.is_mut() && !loan.reserved_at.is_empty() && passes(loan.borrower))
        .map(|(index, _)| index)
        .collect();

    let mut argument_reads = Vec::new();
    for arg in args {
        operand_read(arg, &mut argument_reads);
    }
    let mut before_activation = loans.to_vec();
    for &index in &activated {
        before_activation[index].reserved_at.insert(point);
    }
    check_point(
        body, types, &before_activation, regions, reads, point, &argument_reads, span, is_method, sink, reported,
    );

    for &index in &activated {
        let loan = &loans[index];
        let during: Vec<Loan> = loans
            .iter()
            .filter(|other| !other.capability.is_mut() && loan.reserved_at.contains(&other.created_at))
            .cloned()
            .collect();
        if during.is_empty() {
            continue;
        }
        let place = loan.capability.source_place().expect("a loan has source storage").clone();
        check_point(
            body,
            types,
            &during,
            regions,
            reads,
            point,
            &[(place, Access::Borrow { mutable: true })],
            span,
            is_method,
            sink,
            reported,
        );
    }
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
            let loan_place = loan
                .capability
                .source_place()
                .expect("a loan has source storage");
            if !overlaps(loan_place, place) {
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
            let guard_loan = loan.capability.reference_kind == ReferenceKind::RuntimeGuard;
            if guard_loan {
                if let Access::Borrow { .. } = access {
                    if is_guard_loan(body, types, place) {
                        continue;
                    }
                }
                // A shared guard loan still forbids moving the cell: an
                // ordinary shared loan allows `Read`, but
                // moving inline storage out from under a guard dangles it, so
                // moves conflict here too. (A mutable guard loan already
                // forbids reads and moves.) Writes already conflict for every
                // loan.
                if matches!(access, Access::Move) && !loan.capability.is_mut() {
                    // Fall through to report below (via `refcell` flag).
                } else if matches!(access, Access::Read | Access::Move) {
                    // Mutable case already conflicts via `loan_mutable` below;
                    // keep the same path for the message.
                }
            }
            // `[BRW-3]` — inside the reservation window the borrow is not yet
            // mutable, so shared borrows and reads of the same place pass.
            let reserved = loan.capability.is_mut() && loan.reserved_at.contains(&point);
            let loan_mutable = loan.capability.is_mut() && !reserved;
            let refcell = guard_loan;
            let conflict = match access {
                // While a mutable borrow is live the owner may not read. A
                // shared `RefCell` guard keeps the existing conservative
                // whole-cell read conflict; guard-vs-guard borrows were
                // handled above by the runtime-check exception.
                Access::Read => loan_mutable || refcell,
                // A unique owner may keep its referent at a stable address
                // across a move (Box does), but moving it to a call may also
                // drop it before the live reference's next use. Drop-owning
                // moves therefore conflict with every loan. The existing
                // dropless-value latitude is preserved for types that own no
                // destruction obligation.
                Access::Move => {
                    loan_mutable || refcell || types.needs_drop(place_ty(body, types, place))
                }
                // While any borrow is live the owner may not write.
                Access::Write => true,
                // Two mutable, or one of each, conflict; two shared do not.
                Access::Borrow { mutable } => loan_mutable || *mutable,
            };
            if !conflict {
                continue;
            }

            // `[DIA-14]` — one report per loan and source expression: an
            // assignment to a field that owns storage is a drop and a write
            // at one span (`h.arr = …`), one mistake.
            let key = (
                loan.created_at.block * 4096 + loan.created_at.index,
                ((span.start as usize) << 32) | span.end as usize,
            );
            if !reported.insert(key) {
                continue;
            }

            let name = place_name(body, types, loan_place);
            // A two-phase reservation behaves as shared while its remaining
            // arguments are evaluated, but it is still a mutable borrow for
            // diagnostic identity. If another mutable argument overlaps it,
            // the user wrote two mutable borrows of one place (E3022/B1), not
            // a shared-plus-mutable overlap (E3021/B3).
            let (code, message) = match (loan.capability.is_mut(), access) {
                (true, Access::Borrow { mutable: true }) => (
                    codes::E3022,
                    format!("`{name}` is already mutably borrowed"),
                ),
                (_, Access::Borrow { .. }) => (
                    codes::E3021,
                    format!("`{name}` is borrowed here and mutably borrowed elsewhere"),
                ),
                (true, Access::Read) => (
                    codes::E3021,
                    format!("`{name}` cannot be read while it is mutably borrowed"),
                ),
                (_, Access::Move) => (
                    codes::E3021,
                    format!("`{name}` cannot be moved while it is borrowed"),
                ),
                _ => (
                    codes::E3021,
                    format!("`{name}` cannot be written while it is borrowed"),
                ),
            };

            // `[DIA-3]` — the borrow site, the conflicting access, and the
            // later use that keeps the borrow alive. The last of those is not
            // necessarily the local the borrow was written into: with real
            // regions a loan is kept alive by whatever the reference reached,
            // which may be a copy of it made three lines further down.
            // `[DIA-2]` — never name a temporary at the user. When the
            // borrow lives in one, the advice is about the expression, not
            // about a local they cannot see.
            let (borrower, later) = keeper(body, regions, reads, loan, span);
            // `[EXP-4]`, `[DIA-2]` — the conflicting access is the drop that
            // ends a temporary with its statement, while a view of it lives on
            // (`t = head(make())`). That is a value not living long enough,
            // and the user sees the expression, never the temporary.
            let dropped_temporary = body.local(loan_place.local).name.is_none()
                && matches!(
                    body.blocks[point.block].stmts.get(point.index).map(|stmt| &stmt.kind),
                    Some(StmtKind::Drop { .. })
                );
            if dropped_temporary {
                let mut diagnostic = Diagnostic::error(
                    codes::E3060,
                    body.local(loan_place.local).span,
                    "this temporary is dropped at the end of its statement while it is still borrowed",
                )
                .primary_label("a temporary value, dropped at the end of this statement");
                if let Some(later) = later {
                    diagnostic = diagnostic.secondary(later, "borrow later used here");
                }
                // ODR-065 — the temporary is a handle read out of an object
                // to keep what it points to alive for the borrow.
                let handle = matches!(
                    types.kind(body.local(loan_place.local).ty),
                    ember_types::TyKind::Class(_) | ember_types::TyKind::ClassInterface(_)
                );
                let diagnostic = if handle {
                    diagnostic
                        .help("bind the handle to a variable first (`c = p.child`), and borrow through that")
                        .note("a handle stored in an object can be replaced through another handle, which would free what the borrow points into; the borrow keeps its own copy only to the end of the statement [RC-5]")
                } else {
                    diagnostic
                        .help("bind the value to a variable first, so it lives as long as the borrow")
                        .note("a temporary lives to the end of the statement that makes it [EXP-4]")
                };
                sink.emit_classified(diagnostic);
                continue;
            }
            // `[CTL-2]` survives `for` desugaring as an explicit semantic
            // fact on the synthesized iterator local. A mutable access to the
            // iterable while that local holds this loan is E3020/B2. A
            // manually created iterator remains an ordinary E3021/B3 loan;
            // inferring this distinction from `__it` or source spans would be
            // both fragile and user-spellable.
            let iteration_borrow =
                matches!(access, Access::Write | Access::Borrow { mutable: true })
                    && loan_held_by_for_iterator(body, regions, loan);
            // `[BRW-4]` — when the conflicting access is a method call's
            // receiver autoref, disjoint-field access is defeated by the call:
            // shape B8, which `[DIA-7a]` keys to `E3025` (D-040). A free
            // function's `mut` argument lowers through the same temporary, so
            // the consuming call must be to a method body — otherwise a
            // whole-place `mut` argument misreports as B8 rather than B1.
            let method_root = if loan_mutable && matches!(access, Access::Borrow { mutable: true })
            {
                method_autoref_target(body, point, is_method)
                    .map(|taken| place_name(body, types, &taken))
            } else {
                None
            };
            let (code, message) = if loan.arena_scope {
                (codes::E3096, format!("`{name}` is scoped here"))
            } else if iteration_borrow {
                (
                    codes::E3020,
                    format!("cannot mutate `{name}` while it is borrowed by this loop"),
                )
            } else if let Some(taken) = method_root.as_deref() {
                (
                    codes::E3025,
                    format!("cannot call a method on `{taken}` while `{name}` is mutably borrowed"),
                )
            } else {
                (code, message)
            };
            let kind = if loan.capability.is_mut() {
                "mutable "
            } else {
                ""
            };
            let indexed_conflict = code == codes::E3022
                && loan_place
                    .projection
                    .iter()
                    .any(|p| matches!(p, Projection::Index(_) | Projection::ConstIndex(_)))
                && place
                    .projection
                    .iter()
                    .any(|p| matches!(p, Projection::Index(_) | Projection::ConstIndex(_)));
            let exact_mutable_conflict = code == codes::E3022 && loan_place == place;
            let owner = body
                .local(loan_place.local)
                .name
                .as_deref()
                .unwrap_or("array");
            let mut diagnostic = Diagnostic::error(code, span, message)
                .primary_label(if iteration_borrow {
                    "mutable borrow here"
                } else {
                    "conflicting access here"
                })
                .secondary(
                    loan.span,
                    if iteration_borrow {
                        format!("`{name}` borrowed here for the whole loop")
                    } else {
                        format!("{kind}borrow of `{name}` starts here")
                    },
                );
            if !iteration_borrow && let Some(later) = later {
                diagnostic = diagnostic.secondary(later, "borrow later used here");
            }
            if iteration_borrow {
                sink.emit_classified(
                    diagnostic
                        .help(
                            "collect the indices or values first, then mutate; use `retain` or \
                             `drain` when the operation fits",
                        )
                        .help(
                            "or iterate over a fixed index range and mutate only after taking \
                             the current value",
                        )
                        .note("a `for` loop borrows the iterable until the loop ends (CTL-2)"),
                );
                continue;
            }
            sink.emit_classified(
                diagnostic
                    .help(match (
                        &borrower,
                        loan.arena_scope,
                        method_root.as_deref(),
                        indexed_conflict,
                        exact_mutable_conflict,
                    ) {
                        (_, true, _, _, _) => String::from(
                            "allocate from `scope` instead, or take this allocation before opening the scope",
                        ),
                        // `[DIA-7a]` shape B8's help: the call takes all of
                        // `self`, so the fix is structural, not a shorter borrow.
                        (_, _, Some(_), _, _) => String::from(
                            "inline the field access, take the two fields as separate \
                             parameters, or split the method",
                        ),
                        // ODR-068 — two elements chosen at run time are
                        // `get_pair_mut`'s; a split is the structural fix.
                        (_, _, _, true, _) => format!(
                            "use `{owner}.get_pair_mut(i, j)` for two elements at once (`None` when \
                             they are the same), or `{owner}.as_mut_span().split_at(k)` for two \
                             non-overlapping mutable spans; `swap`, `chunks_mut` and `iter_mut` cover \
                             other patterns"
                        ),
                        (_, _, _, _, true) => String::from(
                            "use one mutable access rather than borrowing the same place twice; \
                             split the owner's mutable view with `split_at` when the intended operands are \
                             disjoint parts of it"
                        ),
                        (Some(name), _, _, _, _) => format!(
                            "end the borrow before this: `{name}` is what keeps it alive, so \
                             shorten its last use or put it in a block of its own"
                        ),
                        (None, _, _, _, _) => format!(
                            "bind the borrow of `{name}` to a local and finish with it before \
                             this line, or copy the value out first",
                            name = name
                        ),
                    })
                    .note(if loan.arena_scope {
                        "a ScopedArena holds its parent's mutable borrow for the scope's whole region (ARN-6)"
                    } else if method_root.is_some() {
                        "a method takes all of `self`, so disjoint fields do not stay disjoint across a call (BRW-4)"
                    } else if indexed_conflict {
                        "computed indices may overlap; a structural split proves non-overlap (BRW-5)"
                    } else {
                        "a borrow lasts until its last use, not to the end of the scope (BRW-2)"
                    }),
            );
        }
    }
}

/// `[CTL-2]`, shape B2 — whether a loan is retained by the compiler-created
/// iterator for a source `for` loop. Region propagation, rather than a direct
/// local equality check, is required because `values.as_span().iter()` carries
/// the owner's loan through both view constructors before reaching `__it`.
fn loan_held_by_for_iterator(body: &Body, regions: &Regions, loan: &Loan) -> bool {
    regions
        .holders(loan.capability.validity_interval.0)
        .iter()
        .any(|holder| body.for_iterators.contains(holder))
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
) -> (Option<String>, Option<Span>) {
    let mut best: Option<(LocalId, Span)> = None;
    for holder in regions.holders(loan.capability.validity_interval.0) {
        if body.local(*holder).name.is_none() {
            continue;
        }
        let Some(spans) = reads.get(holder) else {
            continue;
        };
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
        Some((holder, span)) => (body.local(holder).name.clone(), Some(span)),
        // No later read: the borrow is kept alive by something else — a loop
        // back edge, or the return slot — and the local it was written into is
        // still the honest thing to name.
        None => (body.local(loan.borrower).name.clone(), None),
    }
}

/// Every point at which a local's value is read, by the span of the statement
/// that reads it.
fn collect_reads(body: &Body) -> HashMap<LocalId, Vec<Span>> {
    let mut reads: HashMap<LocalId, Vec<Span>> = HashMap::new();
    let record = |accesses: Vec<(Place, Access)>, span: Span, reads: &mut HashMap<_, Vec<_>>| {
        for (place, access) in accesses {
            // A write through a reference reads the reference itself.
            let reading = matches!(access, Access::Read | Access::Move)
                || place
                    .projection
                    .iter()
                    .any(|p| matches!(p, Projection::Deref));
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
                if let ember_mir::AssertKind::Panic { message } = msg {
                    operand_read(message, &mut accesses);
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
                return false;
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
    let mut out = decl
        .name
        .clone()
        .unwrap_or_else(|| format!("_{}", place.local.0));
    let mut ty = decl.ty;
    let mut at = 0;
    while at < place.projection.len() {
        // IX.1 — Box auto-deref is represented internally as its private
        // pointer field followed by a dereference. Diagnostics must name the
        // source-level owner (`boxed`), never expose `boxed.value` as if that
        // private implementation field were valid Ember syntax.
        if matches!(place.projection.get(at), Some(Projection::Field(0)))
            && matches!(place.projection.get(at + 1), Some(Projection::Deref))
        {
            if let TyKind::Struct(id) = types.kind(ty) {
                let def = types.struct_def(*id);
                if let Some((name, args)) = &def.origin {
                    if name.is("Box") && args.len() == 1 {
                        ty = args[0];
                        at += 2;
                        continue;
                    }
                }
            }
        }
        let projection = &place.projection[at];
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
        at += 1;
    }
    out
}

fn field_name(types: &TypeTable, ty: Ty, index: usize) -> Option<String> {
    match types.kind(ty) {
        TyKind::Struct(id) => types
            .struct_def(*id)
            .fields
            .get(index)
            .map(|f| f.name.to_string()),
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
        TyKind::Ptr { inner, .. } => Some(*inner),
        _ => None,
    }
}

fn pointee_ty(types: &TypeTable, ty: Ty) -> Option<Ty> {
    match types.kind(ty) {
        TyKind::Ref { inner, .. } | TyKind::Ptr { inner, .. } => Some(*inner),
        _ => None,
    }
}

/// The view parameters whose own slot a loan still live at a `return` points
/// into: a reference to the parameter's copy, not to what it views (`E3060`).
fn own_slot_returns(body: &Body, types: &TypeTable, loans: &[Loan], regions: &Regions) -> HashSet<LocalId> {
    let mut found = HashSet::new();
    for (block_index, block) in body.blocks.iter().enumerate() {
        if !matches!(block.terminator, Terminator::Return) {
            continue;
        }
        let point = Point { block: block_index, index: block.stmts.len() };
        for loan in in_scope(loans, regions, point) {
            let Some(place) = loan.capability.source_place() else { continue };
            if loan.capability.must_not_outlive_storage()
                && body.local(place.local).kind == LocalKind::Arg
                && types.is_view(body.local(place.local).ty)
                && !through_indirection(body, types, place)
            {
                found.insert(place.local);
            }
        }
    }
    found
}

/// Whether a place reaches its memory through a borrowed view (a `Deref` of a
/// `ref`, or an element of a `Span` or `str`): memory the view's regions
/// cover. An owned `Array` or `Box` met before any such view (a `Box` is a
/// struct over a raw pointer, and a raw pointer is not a view) is the root's
/// own storage, which a by-copy or `owned` parameter frees or shares without a
/// loan (`[BRW-8]`).
fn through_indirection(body: &Body, types: &TypeTable, place: &Place) -> bool {
    (0..place.projection.len()).any(|k| {
        let base = place_ty(body, types, &Place { local: place.local, projection: place.projection[..k].to_vec() });
        match place.projection[k] {
            Projection::Deref => matches!(types.kind(base), TyKind::Ref { .. }),
            Projection::Index(_) | Projection::ConstIndex(_) => {
                matches!(types.kind(base), TyKind::Span { .. } | TyKind::Str)
            }
            _ => false,
        }
    })
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
            (
                Projection::Index(_) | Projection::ConstIndex(_) | Projection::Column(_),
                TyKind::Ptr { inner, .. },
            ) => ty = *inner,
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
        Operand::Copy(p) => out.push((p.clone(), Access::Read)),
        Operand::Move(p) => out.push((p.clone(), Access::Move)),
        Operand::Const(_) => {}
    }
}

fn project_place(place: &Place, path: &[Projection]) -> Place {
    let mut projected = place.clone();
    projected.projection.extend_from_slice(path);
    projected
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
        // An enum tag read has no payload borrow. The region analysis likewise
        // treats it as control flow rather than a whole-value field access.
        Rvalue::Discriminant(_) => {}
        Rvalue::Ref { place, mutable } => {
            out.push((place.clone(), Access::Borrow { mutable: *mutable }))
        }
    }
}

#[cfg(test)]
mod callable_region_metadata_tests {
    use super::*;
    use ember_mir::{BasicBlock, BasicBlockId, CallableAccessSummary, LocalDecl};

    fn empty_body() -> Body {
        let (_, common) = TypeTable::new();
        let span = Span::new(ember_span::FileId(0), 0, 1);
        Body {
            name: "empty".to_string(),
            symbol: "empty".to_string(),
            is_unsafe: false,
            abi: None,
            locals: vec![LocalDecl {
                ty: common.void,
                kind: LocalKind::Return,
                name: None,
                span,
            }],
            blocks: vec![BasicBlock {
                stmts: Vec::new(),
                terminator: Terminator::Return,
                terminator_span: span,
            }],
            arg_count: 0,
            param_modes: Vec::new(),
            span,
            borrows: None,
            sources: Vec::new(),
            is_lambda: false,
            emit_if_used: false,
            borrowed_params: Vec::new(),
            for_iterators: Vec::new(),
            callable_regions: None,
            closure_environment: None,
            closure_captures_by_move: false,
            class_owner: None,
            class_virtual_slot: None,
            is_abstract: false,
            elided_accesses: Vec::new(),
            hoisted_accesses: Vec::new(),
        }
    }

    #[test]
    fn installed_callable_metadata_reverifies_against_its_body() {
        let (types, _) = TypeTable::new();
        let mut bodies = vec![empty_body()];
        let mut sink = Sink::new();
        check_all(&mut bodies, &types, &mut sink);
        assert!(verify_callable_regions_all(&bodies, &types).is_empty());
    }

    #[test]
    fn malformed_closure_environment_identity_is_rejected() {
        let (types, common) = TypeTable::new();
        let mut body = empty_body();
        body.arg_count = 1;
        body.param_modes = vec![ember_mir::ParameterMode::Borrow];
        body.locals.push(LocalDecl {
            ty: common.i32,
            kind: LocalKind::Arg,
            name: Some("env".to_string()),
            span: Span::DUMMY,
        });
        // The environment record must name the actual first parameter's
        // nominal struct, not an arbitrary compiler-internal id.
        body.closure_environment = Some(ember_types::StructId(0));
        let violations = verify_closure_environments(&[body], &types);
        assert!(
            violations
                .iter()
                .any(|violation| violation.message.contains("closure-environment identity")),
            "malformed closure identity crossed the verifier: {violations:?}"
        );
    }

    #[test]
    fn owned_capture_marker_without_an_environment_is_rejected() {
        let (types, _) = TypeTable::new();
        let mut body = empty_body();
        body.closure_captures_by_move = true;
        let violations = verify_closure_environments(&[body], &types);
        assert!(
            violations
                .iter()
                .any(|violation| violation.message.contains("owned-capture marker")),
            "orphaned owned-capture marker crossed the verifier: {violations:?}"
        );
    }

    #[test]
    fn a_validly_fingerprinted_but_false_summary_is_rejected() {
        let (types, _) = TypeTable::new();
        let mut bodies = vec![empty_body()];
        let mut sink = Sink::new();
        check_all(&mut bodies, &types, &mut sink);
        bodies[0].callable_regions = Some(CallableRegionMetadata::new(
            CallableAccessSummary::All,
            None,
        ));
        let violations = verify_callable_regions_all(&bodies, &types);
        assert!(
            violations
                .iter()
                .any(|v| v.message.contains("disagrees with the MIR body"))
        );
    }

    #[test]
    fn a_call_site_rejects_a_summary_naming_an_absent_argument() {
        let (types, _) = TypeTable::new();
        let span = Span::new(ember_span::FileId(0), 0, 1);
        let callee = empty_body();
        let mut caller = empty_body();
        caller.name = "caller".to_string();
        caller.symbol = "caller".to_string();
        caller.blocks = vec![
            BasicBlock {
                stmts: Vec::new(),
                terminator: Terminator::Call {
                    func: FuncRef::Direct {
                        symbol: "empty".to_string(),
                        latebound: false,
                    },
                    args: Vec::new(),
                    dest: Place::local(ember_mir::RETURN_LOCAL),
                    next: BasicBlockId(1),
                },
                terminator_span: span,
            },
            BasicBlock {
                stmts: Vec::new(),
                terminator: Terminator::Return,
                terminator_span: span,
            },
        ];
        let mut bodies = vec![callee, caller];
        let mut sink = Sink::new();
        check_all(&mut bodies, &types, &mut sink);
        bodies[0].callable_regions = Some(CallableRegionMetadata::new(
            CallableAccessSummary::Fields(vec![ParameterFieldAccess {
                argument: 0,
                projection: Vec::new(),
                operations: vec![RegionAccessKind::Read],
            }]),
            None,
        ));
        let violations = verify_callable_regions_all(&bodies, &types);
        assert!(
            violations
                .iter()
                .any(|v| v.message.contains("names absent argument 0"))
        );
    }
}
