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
    Body, Builtin, CallableAccessSummary as CallAccessContract, CallableRegionMetadata, FuncRef,
    LocalId, LocalKind, Operand, ParameterFieldAccess, Place, Projection, RegionAccessKind,
    ResultFieldProvenance, ResultProvenanceSummary, ResultRegionSource, Rvalue, StmtKind,
    Terminator,
};
use ember_types::{Ty, TyKind, TypeTable};
use ember_span::Span;

use crate::facts::{
    AccessPermission, BorrowCapability, ProvenanceRoot, ReferenceKind, StorageIdentity,
};
use crate::regions::{
    CallRegionContract, CallResultContract, Elision, Origin, Point, Regions,
};

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
    Borrow { mutable: bool },
}

pub fn check_all(bodies: &mut [Body], types: &TypeTable, sink: &mut Sink) {
    // `[LT-1]` is a property of the *callee's* signature, so a call in one
    // body is read against another's. The table is built once.
    let mut signatures: HashMap<String, Elision> = HashMap::new();
    for body in &*bodies {
        signatures.insert(body.symbol.clone(), elision_of(body, types));
    }
    // `[BRW-4]` — a method's first parameter is its `self` receiver, so a
    // direct call to one of these bodies is a method call. Read once here
    // because the conflict is reported while checking the *caller's* body.
    let mut methods: HashSet<String> = HashSet::new();
    for body in &*bodies {
        if is_method_body(body) {
            methods.insert(body.symbol.clone());
        }
    }
    let inferred = infer_callable_summaries(bodies, types, &signatures);
    for body in &mut *bodies {
        let contract = inferred
            .get(&body.symbol)
            .expect("callable summary inference omitted a MIR body");
        body.callable_regions = Some(metadata_from_contract(contract));
    }
    // Borrow checking consumes the installed MIR/interface metadata, not the
    // temporary inference table. That makes the artifact a real producer /
    // consumer boundary rather than a duplicate cache beside the analysis.
    let summaries = contracts_from_metadata(bodies, &signatures);
    let call_contract = |func: &FuncRef| contract_for(func, &summaries, &signatures);
    let invalid_result_bodies =
        infer_invalid_result_bodies(bodies, types, &call_contract);
    let is_method = |func: &FuncRef| match func {
        FuncRef::Direct { symbol } => methods.contains(symbol.as_str()),
        _ => false,
    };
    for body in &*bodies {
        check_body(
            body,
            types,
            &call_contract,
            &invalid_result_bodies,
            &is_method,
            sink,
        );
    }
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
                CallRegionContract { access: metadata.access.clone(), result },
            )
        })
        .collect()
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
    let mut violations = Vec::new();
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
        for body in bodies {
            let regions = Regions::infer(body, types, &call_contract);
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

/// The pre-0.9.5 whole-result relation, retained both for one-region results
/// and as the mandatory conservative fallback for calls with no exact
/// field-to-source summary.
fn legacy_elision(func: &FuncRef, signatures: &HashMap<String, Elision>) -> Elision {
    match func {
        FuncRef::Direct { symbol } => signatures
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
                | Builtin::ArraySplitAtMut { .. }
                | Builtin::SpanSplitAt { .. }
                | Builtin::SpanReborrow
                | Builtin::SpanSharedReborrow
                | Builtin::SpanChunksNew { .. }
                | Builtin::SpanIterNext { .. }
                | Builtin::SpanChunksNext { .. },
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
    }
}

fn conservative_contract(elision: Elision) -> CallRegionContract {
    CallRegionContract {
        access: CallAccessContract::All,
        result: CallResultContract::Legacy(elision),
    }
}

fn contract_for(
    func: &FuncRef,
    summaries: &HashMap<String, CallRegionContract>,
    signatures: &HashMap<String, Elision>,
) -> CallRegionContract {
    if let FuncRef::Direct { symbol } = func
        && let Some(summary) = summaries.get(symbol.as_str())
    {
        return summary.clone();
    }
    if let Some(contract) = builtin_contract(func) {
        return contract;
    }
    conservative_contract(legacy_elision(func, signatures))
}

/// Compiler-known calls whose multi-field result provenance is part of their
/// existing semantic contract. Both halves of a split borrow the same source
/// view even though `[BRW-5]` gives them disjoint storage identities.
fn builtin_contract(func: &FuncRef) -> Option<CallRegionContract> {
    let FuncRef::Builtin {
        which: Builtin::ArraySplitAtMut { .. } | Builtin::SpanSplitAt { .. },
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
        let mut next = HashMap::new();
        for body in bodies {
            let regions = Regions::infer(body, types, &contract);
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
        let whole_value_move = regions.local_regions(local).len() > 1
            && !body.borrowed_params.contains(&local);
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
    }
}

fn inferred_result_summary(
    body: &Body,
    types: &TypeTable,
    regions: &Regions,
) -> Option<ResultProvenanceSummary> {
    let result_slots = regions.local_regions(ember_mir::RETURN_LOCAL);
    if result_slots.len() < 2
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
                    && regions.origins(result.region).contains(&Origin::Param(local))
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
) -> HashSet<String> {
    let known: HashSet<&str> = bodies.iter().map(|body| body.symbol.as_str()).collect();
    let regions: Vec<Regions> = bodies
        .iter()
        .map(|body| Regions::infer(body, types, call_contract))
        .collect();
    let mut invalid = HashSet::new();

    for (body, regions) in bodies.iter().zip(&regions) {
        if unresolved_multi_result_calls(body, regions, call_contract).any(|func| {
            !matches!(
                func,
                FuncRef::Direct { symbol } if known.contains(symbol.as_str())
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
                    FuncRef::Direct { symbol } if invalid.contains(symbol.as_str())
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
    let return_regions = regions.local_regions(ember_mir::RETURN_LOCAL);
    if return_regions.is_empty() {
        return;
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

    let mut offenders: Vec<LocalId> = return_regions
        .iter()
        .flat_map(|slot| regions.origins(slot.region))
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
    offenders.dedup();

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
                FuncRef::Direct { symbol } if invalid_result_bodies.contains(symbol.as_str())
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

/// `[TYP-15]`, `[LT-3]` — a Box has no bounding region, so a view may enter it
/// only when every carried region is static. This check belongs after region
/// inference: spelling the same static view through a local or a zero-input
/// function must not change whether the program is accepted.
fn check_box_storage_regions(
    body: &Body,
    types: &TypeTable,
    regions: &Regions,
    sink: &mut Sink,
) {
    for (block_index, block) in body.blocks.iter().enumerate() {
        let Terminator::Call {
            func: FuncRef::Builtin { which: Builtin::BoxNew { elem, .. }, .. },
            args,
            ..
        } = &block.terminator
        else {
            continue;
        };
        let point = Point { block: block_index, index: block.stmts.len() };
        if !types.is_view(*elem)
            || args
                .first()
                .is_some_and(|value| regions.is_static_operand_at(value, point))
        {
            continue;
        }
        let shown = types.display(*elem);
        sink.emit_classified(
            Diagnostic::error(
                codes::E3063,
                block.terminator_span,
                format!("`{shown}` is a view, so it may not be stored in a Box's contents"),
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

pub fn check(body: &Body, types: &TypeTable, sink: &mut Sink) {
    let is_method = |func: &FuncRef| match func {
        FuncRef::Direct { symbol } => symbol.as_str() == body.symbol.as_str() && is_method_body(body),
        _ => false,
    };
    check_body(
        body,
        types,
        &|_| conservative_contract(Elision::Everything),
        &HashSet::new(),
        &is_method,
        sink,
    );
}

fn check_body(
    body: &Body,
    types: &TypeTable,
    call_contract: &dyn Fn(&FuncRef) -> CallRegionContract,
    invalid_result_bodies: &HashSet<String>,
    is_method: &dyn Fn(&FuncRef) -> bool,
    sink: &mut Sink,
) {
    let regions = Regions::infer(body, types, call_contract);
    check_box_storage_regions(body, types, &regions, sink);
    // Before the loans: a function that hands back a parameter has no loan of
    // its own, and `[LT-1a]` is about exactly that function.
    check_return_regions(body, types, &regions, sink);
    check_multi_result_summary(
        body,
        types,
        &regions,
        call_contract,
        invalid_result_bodies,
        sink,
    );
    let loans = collect_loans(body, types, &regions, call_contract);
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
        let place = loan.capability.source_place().expect("a loan has source storage");
        let root = body.local(place.local);
        if root.kind == LocalKind::Arg
            && (types.is_view(root.ty) || is_named_arena_origin(body, place.local, types))
        {
            continue;
        }
        let name = place_name(body, types, place);
        // The label is about the *owner*, which for `self.n` is `self`.
        let owner = place_name(body, types, &Place::local(place.local));
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
        .filter(|loan| {
            loan.capability
                .source_place()
                .is_some_and(|place| is_guard_loan(body, types, place))
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
    call_contract: &dyn Fn(&FuncRef) -> CallRegionContract,
) -> Vec<Loan> {
    let mut loans = Vec::new();
    for (block_index, block) in body.blocks.iter().enumerate() {
        for (index, stmt) in block.stmts.iter().enumerate() {
            let StmtKind::Assign { place, rvalue } = &stmt.kind else { continue };
            let Rvalue::Ref { place: borrowed, mutable } = rvalue else { continue };
            let created_at = Point { block: block_index, index };
            let Some(region) = regions.loan_region(created_at) else { continue };
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
            let capability = BorrowCapability::statically_checked_reference(
                place_ty(body, types, borrowed),
                provenance_root(body, borrowed.local),
                borrowed.clone(),
                StorageIdentity::PlaceRoot(borrowed.local),
                region,
                permission,
                reference_kind,
            );
            loans.push(Loan {
                capability,
                borrower: place.local,
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
            let created_at = Point { block: block_index, index: block.stmts.len() };
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
    loans
        .iter()
        .filter(|loan| regions.contains(loan.capability.region, point))
        .collect()
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
            let loan_place = loan.capability.source_place().expect("a loan has source storage");
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
            if is_guard_loan(body, types, loan_place) {
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
            let refcell = is_guard_loan(body, types, loan_place);
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

            let key = (loan.created_at.block * 4096 + loan.created_at.index, point.block * 4096 + point.index);
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
                (true, Access::Read) => {
                    (codes::E3021, format!("`{name}` cannot be read while it is mutably borrowed"))
                }
                (_, Access::Move) => {
                    (codes::E3021, format!("`{name}` cannot be moved while it is borrowed"))
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
            let (borrower, later) = keeper(body, regions, reads, loan, span);
            // `[CTL-2]` survives `for` desugaring as an explicit semantic
            // fact on the synthesized iterator local. A mutable access to the
            // iterable while that local holds this loan is E3020/B2. A
            // manually created iterator remains an ordinary E3021/B3 loan;
            // inferring this distinction from `__it` or source spans would be
            // both fragile and user-spellable.
            let iteration_borrow = matches!(access, Access::Write | Access::Borrow { mutable: true })
                && loan_held_by_for_iterator(body, regions, loan);
            // `[BRW-4]` — when the conflicting access is a method call's
            // receiver autoref, disjoint-field access is defeated by the call:
            // shape B8, which `[DIA-7a]` keys to `E3025` (D-040). A free
            // function's `mut` argument lowers through the same temporary, so
            // the consuming call must be to a method body — otherwise a
            // whole-place `mut` argument misreports as B8 rather than B1.
            let method_root = if loan_mutable && matches!(access, Access::Borrow { mutable: true }) {
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
            let kind = if loan.capability.is_mut() { "mutable " } else { "" };
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
                        (_, _, _, true, _) => format!(
                            "use `{owner}.split_at_mut(k)` to obtain two non-overlapping mutable \
                             spans; `chunks_mut`, `iter_mut`, and `columns_mut` cover other \
                             structural access patterns"
                        ),
                        (_, _, _, _, true) => String::from(
                            "use one mutable access rather than borrowing the same place twice; \
                             use `split_at_mut` when the intended operands are disjoint parts of one owner"
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
        .holders(loan.capability.region)
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
    for holder in regions.holders(loan.capability.region) {
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
            span,
            borrows: None,
            borrowed_params: Vec::new(),
            for_iterators: Vec::new(),
            callable_regions: None,
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
        assert!(violations.iter().any(|v| v.message.contains("disagrees with the MIR body")));
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
                    func: FuncRef::Direct { symbol: "empty".to_string() },
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
        assert!(violations.iter().any(|v| v.message.contains("names absent argument 0")));
    }
}
