//! Region variables and their constraint graph (Part XVIII §4.7 steps 1–3).
//!
//! A **region** is the set of program points at which a reference must still
//! be valid. `[LT-5]` says the programmer never writes one; they are inferred,
//! and this module is the inference.
//!
//! The borrow checker used to approximate a region by the liveness of the
//! local holding the borrow. That is exactly right while the reference stays
//! in the local it was created in, and wrong the moment it moves:
//!
//! ```text
//! r: ref mut i32 = ref mut n
//! s: ref mut i32 = r      ## the loan is now held by `s`, and `r` is dead
//! n = 5                   ## …so this looked legal, and is not
//! s = 7
//! ```
//!
//! §4.7 step 2 is what fixes it: an assignment `a = b` between reference-typed
//! places yields `'b: 'a` — *b outlives a* — which as point sets is
//! `points('a) ⊆ points('b)`. The loan's own region is the one the borrow
//! expression produced, so it grows to cover everywhere the reference reaches.
//!
//! ## The shape of it
//!
//! One region variable exists per reference-typed local field slot (§4.7 step
//! 1) and one per borrow expression. A forward value-state fixpoint records
//! which roots can occupy each slot before every MIR point. Backwards
//! field-slot liveness is then projected through that point-sensitive state:
//!
//! * **points** answer "is this loan still live here";
//! * **provenance** answers "which parameter did the value at this point come
//!   from", which is what `[LT-1]`'s elision and `[LT-1a]`'s `@borrows` check;
//! * **accesses** are attributed only to the roots reaching the accessed field
//!   at that point.
//!
//! Reference locals themselves are not reseated (`[TYP-14]`), but `[LT-21]`
//! explicitly permits a borrowed field in a multi-region view to be replaced.
//! Consequently the solver must distinguish the value before that write from
//! the value after it even though both occupy the same compiler region slot.

use std::collections::{HashMap, HashSet};

use ember_mir::{
    AggregateKind, BasicBlockId, Body, CallableAccessSummary as CallAccessContract, FuncRef,
    LocalId, LocalKind, Operand, Place, Projection, RegionAccessKind, ResultProvenanceSummary,
    ResultRegionSource, Rvalue, StmtKind, Terminator,
};
use ember_types::{Ty, TyKind, TypeTable};

/// A point in the CFG: a statement index within a block, where `stmts.len()`
/// is the terminator.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct Point {
    pub block: usize,
    pub index: usize,
}

/// A region variable. Local field slots take the low ids so that
/// `local_regions` is a lookup rather than a map; borrow expressions take the
/// rest.
pub type RegionVid = usize;

/// Where a reference came from, propagated forwards along the same graph the
/// points flow backwards along.
///
/// `[LT-1]` is stated in terms of these: a returned view's region is its
/// parameter's, or the intersection of several, and checking that is asking
/// which parameters a reference can have come from.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum Origin {
    /// A borrow of a parameter, or a copy of a reference passed in as one.
    Param(LocalId),
    /// A borrow of something the frame owns. `[LT-3]`'s `static` region is not
    /// this: a literal borrows nothing and has no origin at all.
    Local(LocalId),
}

/// `[LT-14]`–`[LT-20]` — one compile-time region slot carried by a view
/// value. `projection` names the borrowed field within the nominal value;
/// an empty path is the ordinary one-region case (`ref`, `Span`, `str`).
///
/// This is analysis metadata only. It is not part of `Ty`, layout, symbols,
/// ABI, or generated code (`[LT-30]`, `[LT-31a]`).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ViewRegionSlot {
    pub projection: Vec<Projection>,
    pub region: RegionVid,
}

const ALL_REGION_ACCESSES: &[RegionAccessKind] = &[
    RegionAccessKind::Read,
    RegionAccessKind::Write,
    RegionAccessKind::BorrowShared,
    RegionAccessKind::BorrowMut,
    RegionAccessKind::Move,
    RegionAccessKind::Return,
    RegionAccessKind::Publish,
];

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum CallResultContract {
    /// One-region elision, or the conservative fallback while a multi-region
    /// result relation cannot be inferred.
    Legacy(Elision),
    /// Exact `[LT-22]` field-to-source provenance.
    Fields(ResultProvenanceSummary),
}

/// The complete region information available at one call boundary.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CallRegionContract {
    pub access: CallAccessContract,
    pub result: CallResultContract,
}

impl CallRegionContract {
    pub fn ties(&self, argument: usize) -> bool {
        match &self.result {
            CallResultContract::Legacy(elision) => elision.ties(argument),
            CallResultContract::Fields(summary) => summary.fields.iter().any(|field| {
                field.sources.iter().any(|source| match source {
                    ResultRegionSource::View { argument: source, .. }
                    | ResultRegionSource::Arena { argument: source } => *source == argument,
                })
            }),
        }
    }
}

/// The inferred regions of one function body.
pub struct Regions {
    /// `points[vid]` — where the region is live.
    points: Vec<HashSet<Point>>,
    /// `origins[vid]` — what the reference can have come from.
    origins: Vec<HashSet<Origin>>,
    /// `holders[vid]` — the locals this region reaches, which is `[DIA-3]`'s
    /// "later used here": the borrow is alive at a point because one of these
    /// is still to be read.
    holders: Vec<HashSet<LocalId>>,
    /// Local/result region slots reachable from each region. Unlike
    /// `holders`, this preserves which fields carry a loan for result-summary
    /// inference.
    carried_slots: Vec<HashSet<RegionVid>>,
    /// Operations performed through each region slot. These flow backwards
    /// over the same assignment graph as liveness so a parameter summary sees
    /// accesses through copies, projections, and returned aliases.
    accesses: Vec<HashSet<RegionAccessKind>>,
    /// A result slot reached through a call for which no exact field-level
    /// provenance summary was available. This taint flows forwards with the
    /// value and makes `[LT-22]` reject a wrapper instead of publishing the
    /// old all-fields intersection as if it were inferred precision.
    imprecise_provenance: Vec<bool>,
    /// The region vector of each local. Primitive views have one empty-path
    /// slot; a multi-region `@view struct` has one slot per borrowed field.
    /// Keeping this out of `Ty` preserves nominal identity and runtime erasure.
    local_regions: Vec<Vec<ViewRegionSlot>>,
    /// View values whose destruction has a semantic action tied to their
    /// source (notably `Ref`/`RefMut` guards) must retain their region through
    /// that drop. Ordinary region-erased view destruction needs no such use.
    drop_requires_regions: Vec<bool>,
    /// The region variable produced by the borrow expression at each point.
    loan_region: HashMap<Point, RegionVid>,
    /// Exact field paths used by a verified capturing-closure body for a
    /// synthetic environment borrow at this point. An absent entry is an
    /// ordinary borrow of the complete place. This compiler-only fact keeps
    /// `[LT-42]` precision at the closure boundary without changing reference
    /// layout or granting any general projection-narrowing escape hatch.
    capture_borrow_paths: HashMap<Point, Vec<Vec<Projection>>>,
    /// The possible value roots in every local region slot immediately before
    /// each MIR point. This is compile-time-only `[LT-21]` provenance; it is
    /// deliberately absent from layout, ABI, and generated code.
    values_at: HashMap<Point, Vec<ValueFact>>,
}

#[derive(Clone, Default, PartialEq, Eq, Debug)]
struct ValueFact {
    roots: HashSet<RegionVid>,
    origins: HashSet<Origin>,
    imprecise: bool,
}

impl ValueFact {
    fn merge(&mut self, other: &ValueFact) -> bool {
        let old_roots = self.roots.len();
        self.roots.extend(other.roots.iter().copied());
        let old_origins = self.origins.len();
        self.origins.extend(other.origins.iter().copied());
        let old_imprecise = self.imprecise;
        self.imprecise |= other.imprecise;
        self.roots.len() != old_roots
            || self.origins.len() != old_origins
            || self.imprecise != old_imprecise
    }
}

impl Regions {
    /// §4.7 steps 1–3: allocate the variables, seed them from liveness, and
    /// close the constraint graph.
    ///
    /// Region-slot liveness is computed internally because whole-local
    /// liveness cannot express `[LT-20]`/`[LT-24]` field shortening. The call
    /// contract answers which parameter fields are accessed and how each
    /// result field is tied to the callee's inputs.
    pub fn infer(
        body: &Body,
        types: &TypeTable,
        call_contract: &dyn Fn(&FuncRef) -> CallRegionContract,
    ) -> Regions {
        Self::infer_with_capture_borrow_paths(body, types, call_contract, &HashMap::new())
    }

    /// As [`infer`], but with the verified field paths of synthetic closure
    /// captures supplied by the caller. The map is derived from direct closure
    /// MIR summaries and is intentionally unavailable to source programs.
    pub fn infer_with_capture_borrow_paths(
        body: &Body,
        types: &TypeTable,
        call_contract: &dyn Fn(&FuncRef) -> CallRegionContract,
        capture_borrow_paths: &HashMap<Point, Vec<Vec<Projection>>>,
    ) -> Regions {
        let mut local_regions = Vec::with_capacity(body.locals.len());
        let mut next = 0;
        for decl in &body.locals {
            let mut slots = Vec::new();
            for projection in view_region_paths(types, decl.ty) {
                slots.push(ViewRegionSlot { projection, region: next });
                next += 1;
            }
            local_regions.push(slots);
        }

        let mut loan_region = HashMap::new();
        for (block_index, block) in body.blocks.iter().enumerate() {
            for (index, stmt) in block.stmts.iter().enumerate() {
                let StmtKind::Assign { rvalue: Rvalue::Ref { .. }, .. } = &stmt.kind else {
                    continue;
                };
                loan_region.insert(Point { block: block_index, index }, next);
                next += 1;
            }
        }

        let mut carried_slots = vec![HashSet::new(); next];
        for (region, slots) in carried_slots.iter_mut().enumerate() {
            slots.insert(region);
        }
        let mut regions = Regions {
            points: vec![HashSet::new(); next],
            origins: vec![HashSet::new(); next],
            holders: vec![HashSet::new(); next],
            carried_slots,
            accesses: vec![HashSet::new(); next],
            imprecise_provenance: vec![false; next],
            local_regions,
            drop_requires_regions: body.locals.iter().map(|decl| types.needs_drop(decl.ty)).collect(),
            loan_region,
            capture_borrow_paths: capture_borrow_paths.clone(),
            values_at: HashMap::new(),
        };

        for (index, slots) in regions.local_regions.iter().enumerate() {
            for slot in slots {
                regions.holders[slot.region].insert(LocalId(index as u32));
            }
        }

        // Step 3, the seed: each field slot is live only where that field (or
        // a containing/whole-value place) can next be read. This is the
        // field-sensitive NLL fact `[LT-20]` requires; using whole-local
        // liveness here is the old intersection model.
        let slot_liveness = regions.slot_liveness(body, call_contract);
        for (point, live_slots) in &slot_liveness {
            for region in live_slots {
                regions.points[*region].insert(*point);
            }
        }

        // A view-typed parameter is where provenance starts. `[LT-1]` is
        // stated over exactly these: what the caller owns, which the callee
        // may hand back and may not outlive.
        for (local, _) in body.args() {
            for slot in &regions.local_regions[local.0 as usize] {
                regions.origins[slot.region].insert(Origin::Param(local));
            }
        }

        regions.values_at = regions.infer_value_states(body, types, call_contract);
        regions.apply_value_states(body, call_contract, &slot_liveness);
        regions
    }

    /// The exact field paths of a compiler-generated closure capture borrow.
    /// `None` means this is an ordinary whole-place borrow.
    pub fn capture_borrow_paths(&self, point: Point) -> Option<&[Vec<Projection>]> {
        self.capture_borrow_paths.get(&point).map(Vec::as_slice)
    }

    /// The region of the borrow expression at `point`, if there is one.
    pub fn loan_region(&self, point: Point) -> Option<RegionVid> {
        self.loan_region.get(&point).copied()
    }

    /// The sole region of an ordinary one-region local. Multi-region values
    /// deliberately return `None`; callers that operate on whole values must
    /// use `local_regions` and require every slot.
    pub fn local_region(&self, local: LocalId) -> Option<RegionVid> {
        let slots = &self.local_regions[local.0 as usize];
        (slots.len() == 1).then(|| slots[0].region)
    }

    /// The complete compiler-internal region vector for a local.
    pub fn local_regions(&self, local: LocalId) -> &[ViewRegionSlot] {
        &self.local_regions[local.0 as usize]
    }

    pub fn contains(&self, region: RegionVid, point: Point) -> bool {
        self.points[region].contains(&point)
    }

    pub fn origins(&self, region: RegionVid) -> &HashSet<Origin> {
        &self.origins[region]
    }

    /// The locals a region reaches: the references that keep a loan alive.
    pub fn holders(&self, region: RegionVid) -> &HashSet<LocalId> {
        &self.holders[region]
    }

    /// Whether `from` carries the value represented by `to` after closing the
    /// assignment/call graph.
    pub fn reaches_slot(&self, from: RegionVid, to: RegionVid) -> bool {
        self.carried_slots[from].contains(&to)
    }

    pub fn has_imprecise_provenance(&self, region: RegionVid) -> bool {
        self.imprecise_provenance[region]
    }

    pub fn accesses(&self, region: RegionVid) -> &HashSet<RegionAccessKind> {
        &self.accesses[region]
    }

    /// `[LT-3]` — whether a view operand is proven to carry only the static
    /// region. Constants borrow no runtime storage. A view local is static
    /// exactly when provenance closure found no parameter or local origin for
    /// it; this preserves the fact through bindings and through calls whose
    /// result is not tied to a view argument.
    pub fn is_static_operand_at(&self, operand: &Operand, point: Point) -> bool {
        match operand {
            Operand::Const(_) => true,
            Operand::Copy(place) | Operand::Move(place) => {
                let regions = self.place_regions(place);
                let Some(state) = self.values_at.get(&point) else { return false };
                !regions.is_empty()
                    && regions
                        .iter()
                        .all(|region| state[*region].origins.is_empty())
            }
        }
    }

    /// `[VERIFY-3]` — prove that the region graph consumed every slot named
    /// by installed call metadata and routed every exact result source to its
    /// destination field. This checks the consumer side independently of the
    /// summary/body agreement check in `borrows.rs`.
    pub(crate) fn verify_call_contracts(
        &self,
        body: &Body,
        types: &TypeTable,
        call_contract: &dyn Fn(&FuncRef) -> CallRegionContract,
    ) -> Vec<String> {
        let mut violations = Vec::new();
        for (block_index, block) in body.blocks.iter().enumerate() {
            let Terminator::Call { func, args, dest, .. } = &block.terminator else {
                continue;
            };
            let point = Point { block: block_index, index: block.stmts.len() };
            let contract = call_contract(func);
            match &contract.access {
                CallAccessContract::All => {
                    for (argument, value) in args.iter().enumerate() {
                        for region in self.operand_regions(value) {
                            if !self.contains(region, point) {
                                violations.push(format!(
                                    "bb{block_index}: conservative argument {argument} region is not required at the call"
                                ));
                            }
                        }
                    }
                }
                CallAccessContract::Fields(accesses) => {
                    for access in accesses {
                        let Some(argument) = args.get(access.argument) else {
                            violations.push(format!(
                                "bb{block_index}: callable summary names absent argument {}",
                                access.argument
                            ));
                            continue;
                        };
                        for region in self.operand_regions_at(argument, &access.projection) {
                            if !self.contains(region, point) {
                                violations.push(format!(
                                    "bb{block_index}: argument {} field {:?} is not required at the call",
                                    access.argument, access.projection
                                ));
                            }
                        }
                    }
                }
            }

            let CallResultContract::Fields(summary) = &contract.result else {
                continue;
            };
            let Some(state) = self.values_at.get(&point) else {
                violations.push(format!(
                    "bb{block_index}: callable result has no point-sensitive input state"
                ));
                continue;
            };
            let produced = self.call_result_facts(
                body,
                types,
                state,
                func,
                args,
                dest,
                call_contract,
            );
            for field in &summary.fields {
                let mut result = dest.clone();
                result.projection.extend(field.result_projection.clone());
                let destinations = self.assigned_place_regions(&result);
                if destinations.is_empty() {
                    violations.push(format!(
                        "bb{block_index}: result field {:?} names no destination region slot",
                        field.result_projection
                    ));
                    continue;
                }
                for source in &field.sources {
                    match source {
                        ResultRegionSource::View { argument, projection } => {
                            let Some(value) = args.get(*argument) else {
                                violations.push(format!(
                                    "bb{block_index}: result provenance names absent argument {argument}"
                                ));
                                continue;
                            };
                            let sources = self.operand_regions_at(value, projection);
                            if sources.is_empty() {
                                violations.push(format!(
                                    "bb{block_index}: result provenance argument {argument} field {projection:?} names no source region slot"
                                ));
                            }
                            let source_fact = self.fact_for_operand_at(state, value, projection);
                            for destination in &destinations {
                                let destination_fact = produced.get(destination);
                                let retains_source = destination_fact.is_some_and(|fact| {
                                    source_fact.roots.is_subset(&fact.roots)
                                        && source_fact.origins.is_subset(&fact.origins)
                                        && (!source_fact.imprecise || fact.imprecise)
                                });
                                if !retains_source {
                                    violations.push(format!(
                                        "bb{block_index}: result field {:?} does not retain argument {argument} field {projection:?}",
                                        field.result_projection
                                    ));
                                }
                            }
                        }
                        ResultRegionSource::Arena { argument } => {
                            let Some(Operand::Copy(place) | Operand::Move(place)) =
                                args.get(*argument)
                            else {
                                violations.push(format!(
                                    "bb{block_index}: Arena provenance names non-place argument {argument}"
                                ));
                                continue;
                            };
                            if !is_growing_arena(types, body.local(place.local).ty) {
                                violations.push(format!(
                                    "bb{block_index}: result provenance argument {argument} is not a growing Arena"
                                ));
                                continue;
                            }
                            let origin = match body.local(place.local).kind {
                                LocalKind::Arg => Origin::Param(place.local),
                                _ => Origin::Local(place.local),
                            };
                            for destination in &destinations {
                                if !produced
                                    .get(destination)
                                    .is_some_and(|fact| fact.origins.contains(&origin))
                                {
                                    violations.push(format!(
                                        "bb{block_index}: result field {:?} does not retain Arena argument {argument}",
                                        field.result_projection
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }
        violations
    }

    /// `[LT-21]` — infer the possible value occupying every local region slot
    /// before every MIR point. Assignments replace a fact; CFG joins merge
    /// facts. This is the minimum location sensitivity field replacement
    /// requires and is deliberately independent of runtime representation.
    fn infer_value_states(
        &self,
        body: &Body,
        types: &TypeTable,
        call_contract: &dyn Fn(&FuncRef) -> CallRegionContract,
    ) -> HashMap<Point, Vec<ValueFact>> {
        let mut entry = vec![ValueFact::default(); self.points.len()];
        for (local, _) in body.args() {
            for slot in &self.local_regions[local.0 as usize] {
                entry[slot.region].roots.insert(slot.region);
                entry[slot.region].origins.insert(Origin::Param(local));
            }
        }

        let mut block_in: Vec<Option<Vec<ValueFact>>> = vec![None; body.blocks.len()];
        if !body.blocks.is_empty() {
            block_in[0] = Some(entry);
        }
        let mut changed = true;
        while changed {
            changed = false;
            for (block_index, block) in body.blocks.iter().enumerate() {
                let Some(mut state) = block_in[block_index].clone() else { continue };
                for (index, stmt) in block.stmts.iter().enumerate() {
                    self.transfer_statement(
                        body,
                        &mut state,
                        Point { block: block_index, index },
                        &stmt.kind,
                    );
                }
                self.transfer_terminator(
                    body,
                    types,
                    &mut state,
                    Point { block: block_index, index: block.stmts.len() },
                    &block.terminator,
                    call_contract,
                );
                for successor in region_successors(&block.terminator) {
                    let target = &mut block_in[successor.0 as usize];
                    match target {
                        Some(existing) => {
                            for (old, incoming) in existing.iter_mut().zip(&state) {
                                changed |= old.merge(incoming);
                            }
                        }
                        None => {
                            *target = Some(state.clone());
                            changed = true;
                        }
                    }
                }
            }
        }

        let mut values_at = HashMap::new();
        for (block_index, block) in body.blocks.iter().enumerate() {
            let Some(mut state) = block_in[block_index].clone() else { continue };
            for (index, stmt) in block.stmts.iter().enumerate() {
                let point = Point { block: block_index, index };
                values_at.insert(point, state.clone());
                self.transfer_statement(body, &mut state, point, &stmt.kind);
            }
            let point = Point { block: block_index, index: block.stmts.len() };
            values_at.insert(point, state);
        }
        values_at
    }

    fn transfer_statement(
        &self,
        body: &Body,
        state: &mut [ValueFact],
        point: Point,
        kind: &StmtKind,
    ) {
        match kind {
            StmtKind::Assign { place, rvalue } => {
                let assignments = self.rvalue_facts(body, state, point, place, rvalue);
                self.install_facts(state, &self.assigned_place_regions(place), assignments);
            }
            StmtKind::CheckedBinaryOp { dest, overflow, .. } => {
                self.clear_place_facts(state, dest);
                self.clear_place_facts(state, overflow);
            }
            StmtKind::StorageLive(local) | StmtKind::StorageDead(local) => {
                for slot in &self.local_regions[local.0 as usize] {
                    state[slot.region] = ValueFact::default();
                }
            }
            StmtKind::Drop { .. } | StmtKind::Nop => {}
        }
    }

    fn transfer_terminator(
        &self,
        body: &Body,
        types: &TypeTable,
        state: &mut [ValueFact],
        _point: Point,
        terminator: &Terminator,
        call_contract: &dyn Fn(&FuncRef) -> CallRegionContract,
    ) {
        if let Terminator::Call { func, args, dest, .. } = terminator {
            let assignments = self.call_result_facts(body, types, state, func, args, dest, call_contract);
            self.install_facts(state, &self.assigned_place_regions(dest), assignments);
        }
    }

    fn rvalue_facts(
        &self,
        body: &Body,
        state: &[ValueFact],
        point: Point,
        destination: &Place,
        rvalue: &Rvalue,
    ) -> HashMap<RegionVid, ValueFact> {
        let destinations = self.assigned_place_regions(destination);
        let mut result = HashMap::new();
        match rvalue {
            Rvalue::Ref { place, .. } => {
                if let Some(paths) = self.capture_borrow_paths(point) {
                    let Some(loan) = self.loan_region.get(&point).copied() else {
                        return result;
                    };
                    for path in paths {
                        let source = project_place(place, path);
                        let mut fact = self.fact_for_place(state, &source);
                        fact.roots.insert(loan);
                        if self.deref_base(&source).is_none() {
                            fact.origins.insert(match body.local(source.local).kind {
                                LocalKind::Arg => Origin::Param(source.local),
                                _ => Origin::Local(source.local),
                            });
                        }
                        // A `ref Pair` has slots rooted at `Deref`; route the
                        // selected source field to its corresponding slot
                        // rather than copying a whole-pair fact to every slot.
                        let mut target_path = vec![Projection::Deref];
                        target_path.extend(path.iter().cloned());
                        let target = project_place(destination, &target_path);
                        for destination in self.assigned_place_regions(&target) {
                            result.insert(destination, fact.clone());
                        }
                    }
                    return result;
                }
                let mut fact = self.fact_for_place(state, place);
                let Some(loan) = self.loan_region.get(&point).copied() else { return result };
                fact.roots.insert(loan);
                if self.deref_base(place).is_none() {
                    fact.origins.insert(match body.local(place.local).kind {
                        LocalKind::Arg => Origin::Param(place.local),
                        _ => Origin::Local(place.local),
                    });
                }
                for destination in destinations {
                    result.insert(destination, fact.clone());
                }
            }
            Rvalue::Use(operand) | Rvalue::Cast { operand, .. } => {
                self.map_facts(
                    state,
                    &self.operand_regions(operand),
                    &destinations,
                    &mut result,
                );
            }
            Rvalue::Aggregate { kind, operands } if matches!(kind, AggregateKind::Array) => {
                let mut fact = ValueFact::default();
                for operand in operands {
                    fact.merge(&self.fact_for_operand(state, operand));
                }
                for destination in destinations {
                    result.insert(destination, fact.clone());
                }
            }
            Rvalue::Aggregate { kind, operands } => {
                for (field, operand) in operands.iter().enumerate() {
                    let mut projection = destination.projection.clone();
                    projection.extend(aggregate_field_projection(*kind, field));
                    let field_place = Place { local: destination.local, projection };
                    self.map_facts(
                        state,
                        &self.operand_regions(operand),
                        &self.assigned_place_regions(&field_place),
                        &mut result,
                    );
                }
            }
            Rvalue::Repeat { value, .. } => {
                let fact = self.fact_for_operand(state, value);
                for destination in destinations {
                    result.insert(destination, fact.clone());
                }
            }
            Rvalue::BinaryOp { .. }
            | Rvalue::UnaryOp { .. }
            | Rvalue::Discriminant(_) => {}
        }
        result
    }

    fn call_result_facts(
        &self,
        body: &Body,
        types: &TypeTable,
        state: &[ValueFact],
        func: &FuncRef,
        args: &[Operand],
        destination: &Place,
        call_contract: &dyn Fn(&FuncRef) -> CallRegionContract,
    ) -> HashMap<RegionVid, ValueFact> {
        let destinations = self.assigned_place_regions(destination);
        let mut result = HashMap::new();
        match call_contract(func).result {
            CallResultContract::Fields(summary) => {
                for field in summary.fields {
                    let mut result_place = destination.clone();
                    result_place.projection.extend(field.result_projection);
                    let mut fact = ValueFact::default();
                    for source in field.sources {
                        match source {
                            ResultRegionSource::View { argument, projection } => {
                                if let Some(argument) = args.get(argument) {
                                    fact.merge(&self.fact_for_operand_at(
                                        state,
                                        argument,
                                        &projection,
                                    ));
                                }
                            }
                            ResultRegionSource::Arena { argument } => {
                                if let Some(Operand::Copy(place) | Operand::Move(place)) =
                                    args.get(argument)
                                    && is_growing_arena(types, body.local(place.local).ty)
                                {
                                    fact.origins.insert(match body.local(place.local).kind {
                                        LocalKind::Arg => Origin::Param(place.local),
                                        _ => Origin::Local(place.local),
                                    });
                                }
                            }
                        }
                    }
                    for destination in self.assigned_place_regions(&result_place) {
                        result.insert(destination, fact.clone());
                    }
                }
            }
            CallResultContract::Legacy(tied) => {
                let mut fact = ValueFact::default();
                for (index, argument) in args.iter().enumerate() {
                    if !tied.ties(index) {
                        continue;
                    }
                    let source = self.fact_for_operand(state, argument);
                    if source.roots.is_empty()
                        && source.origins.is_empty()
                        && let Operand::Copy(place) | Operand::Move(place) = argument
                        && is_growing_arena(types, body.local(place.local).ty)
                    {
                        fact.origins.insert(match body.local(place.local).kind {
                            LocalKind::Arg => Origin::Param(place.local),
                            _ => Origin::Local(place.local),
                        });
                    } else {
                        fact.merge(&source);
                    }
                }
                fact.imprecise |= destinations.len() > 1;
                for destination in destinations {
                    result.insert(destination, fact.clone());
                }
            }
        }
        result
    }

    fn install_facts(
        &self,
        state: &mut [ValueFact],
        destinations: &[RegionVid],
        mut assignments: HashMap<RegionVid, ValueFact>,
    ) {
        for destination in destinations {
            state[*destination] = assignments.remove(destination).unwrap_or_default();
        }
    }

    fn clear_place_facts(&self, state: &mut [ValueFact], place: &Place) {
        for destination in self.assigned_place_regions(place) {
            state[destination] = ValueFact::default();
        }
    }

    fn map_facts(
        &self,
        state: &[ValueFact],
        sources: &[RegionVid],
        destinations: &[RegionVid],
        result: &mut HashMap<RegionVid, ValueFact>,
    ) {
        if sources.len() == destinations.len() {
            for (source, destination) in sources.iter().zip(destinations) {
                result
                    .entry(*destination)
                    .or_default()
                    .merge(&state[*source]);
            }
        } else {
            let mut fact = ValueFact::default();
            for source in sources {
                fact.merge(&state[*source]);
            }
            for destination in destinations {
                result.entry(*destination).or_default().merge(&fact);
            }
        }
    }

    fn fact_for_place(&self, state: &[ValueFact], place: &Place) -> ValueFact {
        let mut fact = ValueFact::default();
        for region in self.place_regions(place) {
            fact.merge(&state[region]);
        }
        fact
    }

    fn fact_for_operand(&self, state: &[ValueFact], operand: &Operand) -> ValueFact {
        match operand {
            Operand::Copy(place) | Operand::Move(place) => self.fact_for_place(state, place),
            Operand::Const(_) => ValueFact::default(),
        }
    }

    fn fact_for_operand_at(
        &self,
        state: &[ValueFact],
        operand: &Operand,
        relative_projection: &[Projection],
    ) -> ValueFact {
        match operand {
            Operand::Copy(place) | Operand::Move(place) => {
                let mut projected = place.clone();
                projected.projection.extend_from_slice(relative_projection);
                self.fact_for_place(state, &projected)
            }
            Operand::Const(_) => ValueFact::default(),
        }
    }

    /// Project field liveness and operations through the value present at the
    /// same MIR point. Historical assignments therefore cannot keep their old
    /// source alive or leak it into a callable result/access summary.
    fn apply_value_states(
        &mut self,
        body: &Body,
        call_contract: &dyn Fn(&FuncRef) -> CallRegionContract,
        slot_liveness: &HashMap<Point, HashSet<RegionVid>>,
    ) {
        let mut slot_holders = vec![None; self.points.len()];
        for (local, slots) in self.local_regions.iter().enumerate() {
            for slot in slots {
                slot_holders[slot.region] = Some(LocalId(local as u32));
            }
        }

        for (point, live_slots) in slot_liveness {
            let Some(state) = self.values_at.get(point) else { continue };
            for slot in live_slots {
                let holder = slot_holders[*slot];
                for root in &state[*slot].roots {
                    self.points[*root].insert(*point);
                    if let Some(holder) = holder {
                        self.holders[*root].insert(holder);
                    }
                    self.origins[*root].extend(state[*slot].origins.iter().copied());
                }
            }
        }

        self.accesses.iter_mut().for_each(HashSet::clear);
        for (point, place, operation) in self.access_events(body, call_contract) {
            let Some(state) = self.values_at.get(&point) else { continue };
            for root in self.fact_for_place(state, &place).roots {
                self.accesses[root].insert(operation);
            }
        }

        // Result summaries are facts at actual Return points, not unions of
        // every historical assignment ever made to the return slot.
        let result_slots = self.local_regions[ember_mir::RETURN_LOCAL.0 as usize].clone();
        for result in result_slots {
            let mut fact = ValueFact::default();
            for (block_index, block) in body.blocks.iter().enumerate() {
                if !matches!(block.terminator, Terminator::Return) {
                    continue;
                }
                let point = Point { block: block_index, index: block.stmts.len() };
                if let Some(state) = self.values_at.get(&point) {
                    fact.merge(&state[result.region]);
                }
            }
            self.origins[result.region] = fact.origins.clone();
            self.imprecise_provenance[result.region] = fact.imprecise;
            for root in fact.roots {
                self.carried_slots[root].insert(result.region);
            }
        }
    }

    fn access_events(
        &self,
        body: &Body,
        call_contract: &dyn Fn(&FuncRef) -> CallRegionContract,
    ) -> Vec<(Point, Place, RegionAccessKind)> {
        let mut events = Vec::new();
        for (block_index, block) in body.blocks.iter().enumerate() {
            for (index, stmt) in block.stmts.iter().enumerate() {
                let point = Point { block: block_index, index };
                match &stmt.kind {
                    StmtKind::Assign { place, rvalue } => {
                        self.rvalue_access_events(point, rvalue, &mut events);
                        if self.assigned_place_regions(place).is_empty() {
                            events.push((point, place.clone(), RegionAccessKind::Write));
                        }
                    }
                    StmtKind::CheckedBinaryOp { lhs, rhs, .. } => {
                        self.operand_access_event(point, lhs, &mut events);
                        self.operand_access_event(point, rhs, &mut events);
                    }
                    StmtKind::Drop { place, .. }
                        if self.drop_requires_regions[place.local.0 as usize] =>
                    {
                        events.push((point, place.clone(), RegionAccessKind::Read));
                    }
                    StmtKind::StorageLive(_)
                    | StmtKind::StorageDead(_)
                    | StmtKind::Drop { .. }
                    | StmtKind::Nop => {}
                }
            }

            let point = Point { block: block_index, index: block.stmts.len() };
            match &block.terminator {
                Terminator::SwitchInt { discr, .. } => {
                    self.operand_access_event(point, discr, &mut events)
                }
                Terminator::Call { func, args, .. } => match call_contract(func).access {
                    CallAccessContract::All => {
                        for argument in args {
                            for operation in ALL_REGION_ACCESSES {
                                self.operand_access_event_with(
                                    point,
                                    argument,
                                    *operation,
                                    &mut events,
                                );
                            }
                        }
                    }
                    CallAccessContract::Fields(accesses) => {
                        for access in accesses {
                            let Some(argument) = args.get(access.argument) else { continue };
                            if let Operand::Copy(place) | Operand::Move(place) = argument {
                                let mut projected = place.clone();
                                projected.projection.extend(access.projection);
                                for operation in access.operations {
                                    events.push((point, projected.clone(), operation));
                                }
                            }
                        }
                    }
                },
                Terminator::Assert { cond, msg, .. } => {
                    self.operand_access_event(point, cond, &mut events);
                    if let ember_mir::AssertKind::RefCellBorrow { file, line } = msg {
                        self.operand_access_event(point, file, &mut events);
                        self.operand_access_event(point, line, &mut events);
                    }
                }
                Terminator::Return => {
                    for slot in &self.local_regions[ember_mir::RETURN_LOCAL.0 as usize] {
                        events.push((
                            point,
                            Place {
                                local: ember_mir::RETURN_LOCAL,
                                projection: slot.projection.clone(),
                            },
                            RegionAccessKind::Return,
                        ));
                    }
                }
                Terminator::Goto(_) | Terminator::Unreachable => {}
            }
        }
        events
    }

    fn rvalue_access_events(
        &self,
        point: Point,
        rvalue: &Rvalue,
        events: &mut Vec<(Point, Place, RegionAccessKind)>,
    ) {
        match rvalue {
            Rvalue::Use(operand) | Rvalue::UnaryOp { operand, .. } | Rvalue::Cast { operand, .. } => {
                self.operand_access_event(point, operand, events)
            }
            Rvalue::BinaryOp { lhs, rhs, .. } => {
                self.operand_access_event(point, lhs, events);
                self.operand_access_event(point, rhs, events);
            }
            Rvalue::Aggregate { operands, .. } => {
                for operand in operands {
                    self.operand_access_event(point, operand, events);
                }
            }
            Rvalue::Repeat { value, .. } => self.operand_access_event(point, value, events),
            // The discriminant is representation metadata, not a borrowed
            // payload read. Access summaries therefore reflect only the field
            // projection reached on the selected control-flow edge.
            Rvalue::Discriminant(_) => {}
            Rvalue::Ref { place, mutable } => {
                let operation = if *mutable {
                    RegionAccessKind::BorrowMut
                } else {
                    RegionAccessKind::BorrowShared
                };
                if let Some(paths) = self.capture_borrow_paths(point) {
                    for path in paths {
                        events.push((point, project_place(place, path), operation));
                    }
                } else {
                    events.push((point, place.clone(), operation));
                }
            }
        }
    }

    fn operand_access_event(
        &self,
        point: Point,
        operand: &Operand,
        events: &mut Vec<(Point, Place, RegionAccessKind)>,
    ) {
        self.operand_access_event_with(
            point,
            operand,
            match operand {
                Operand::Move(_) => RegionAccessKind::Move,
                Operand::Copy(_) => RegionAccessKind::Read,
                Operand::Const(_) => return,
            },
            events,
        );
    }

    fn operand_access_event_with(
        &self,
        point: Point,
        operand: &Operand,
        operation: RegionAccessKind,
        events: &mut Vec<(Point, Place, RegionAccessKind)>,
    ) {
        if let Operand::Copy(place) | Operand::Move(place) = operand {
            events.push((point, place.clone(), operation));
        }
    }

    /// Region slots selected by a place. A whole-value place selects every
    /// slot; a field projection selects only slots at or below that field; a
    /// dereference/index past a view leaf still uses that leaf's slot.
    pub fn place_regions(&self, place: &Place) -> Vec<RegionVid> {
        let slots = &self.local_regions[place.local.0 as usize];
        // `[LT-34]` — a legacy one-region view keeps its old validity
        // boundary even when the borrowed field is only an internal anchor
        // (Arena collections are the important case). Field-sensitive
        // projection starts only once the nominal value actually carries
        // more than one independent region slot.
        if slots.len() == 1 {
            return vec![slots[0].region];
        }
        slots
            .iter()
            .filter(|slot| paths_overlap(&slot.projection, &place.projection))
            .map(|slot| slot.region)
            .collect()
    }

    /// Region slots whose *view values* are replaced by assignment to this
    /// place. A dereference or index beyond a view leaf writes the referent,
    /// not the reference/span value, and therefore is not a provenance flow.
    fn assigned_place_regions(&self, place: &Place) -> Vec<RegionVid> {
        self.local_regions[place.local.0 as usize]
            .iter()
            .filter(|slot| path_is_prefix(&place.projection, &slot.projection))
            .map(|slot| slot.region)
            .collect()
    }

    fn operand_regions(&self, operand: &Operand) -> Vec<RegionVid> {
        match operand {
            Operand::Copy(place) | Operand::Move(place) => self.place_regions(place),
            // `[LT-3]` — a literal has the static region, which outlives
            // everything and constrains nothing.
            Operand::Const(_) => Vec::new(),
        }
    }

    fn operand_regions_at(
        &self,
        operand: &Operand,
        relative_projection: &[Projection],
    ) -> Vec<RegionVid> {
        match operand {
            Operand::Copy(place) | Operand::Move(place) => {
                let mut projected = place.clone();
                projected.projection.extend_from_slice(relative_projection);
                self.place_regions(&projected)
            }
            Operand::Const(_) => Vec::new(),
        }
    }

    /// `[LT-20]`, `[LT-24]` — backwards liveness over region slots rather
    /// than whole locals. This permits `pair.left` to die while `pair.right`
    /// remains usable, without changing ordinary local liveness used by the
    /// ownership passes.
    fn slot_liveness(
        &self,
        body: &Body,
        call_contract: &dyn Fn(&FuncRef) -> CallRegionContract,
    ) -> HashMap<Point, HashSet<RegionVid>> {
        let mut live_out: HashMap<usize, HashSet<RegionVid>> = HashMap::new();
        let mut changed = true;
        while changed {
            changed = false;
            for index in (0..body.blocks.len()).rev() {
                let mut out = HashSet::new();
                for successor in region_successors(&body.blocks[index].terminator) {
                    if let Some(entry) =
                        self.slot_live_in(body, successor.0 as usize, &live_out, call_contract)
                    {
                        out.extend(entry);
                    }
                }
                if live_out.get(&index).map(|old| old != &out).unwrap_or(true) {
                    live_out.insert(index, out);
                    changed = true;
                }
            }
        }

        let mut points = HashMap::new();
        for (block_index, block) in body.blocks.iter().enumerate() {
            let mut live = live_out.get(&block_index).cloned().unwrap_or_default();
            self.terminator_liveness(&block.terminator, &mut live, call_contract);
            points.insert(
                Point { block: block_index, index: block.stmts.len() },
                live.clone(),
            );
            for (index, stmt) in block.stmts.iter().enumerate().rev() {
                let point = Point { block: block_index, index };
                self.statement_liveness(point, &stmt.kind, &mut live);
                points.insert(point, live.clone());
            }
        }
        points
    }

    fn slot_live_in(
        &self,
        body: &Body,
        block_index: usize,
        live_out: &HashMap<usize, HashSet<RegionVid>>,
        call_contract: &dyn Fn(&FuncRef) -> CallRegionContract,
    ) -> Option<HashSet<RegionVid>> {
        let block = body.blocks.get(block_index)?;
        let mut live = live_out.get(&block_index).cloned().unwrap_or_default();
        self.terminator_liveness(&block.terminator, &mut live, call_contract);
        for (index, stmt) in block.stmts.iter().enumerate().rev() {
            self.statement_liveness(
                Point { block: block_index, index },
                &stmt.kind,
                &mut live,
            );
        }
        Some(live)
    }

    fn statement_liveness(
        &self,
        point: Point,
        kind: &StmtKind,
        live: &mut HashSet<RegionVid>,
    ) {
        match kind {
            StmtKind::Assign { place, rvalue } => {
                self.write_place_liveness(place, live);
                self.rvalue_liveness(point, rvalue, live);
            }
            StmtKind::CheckedBinaryOp { dest, overflow, lhs, rhs, .. } => {
                self.write_place_liveness(dest, live);
                self.write_place_liveness(overflow, live);
                self.operand_liveness(lhs, live);
                self.operand_liveness(rhs, live);
            }
            // `[LT-29]`: ordinary view destruction does not access sources.
            // A compiler-known guard is the exception already required by
            // `[CELL-7]`: its destructor releases runtime borrow state.
            StmtKind::Drop { place, .. }
                if self.drop_requires_regions[place.local.0 as usize] =>
            {
                self.read_place_liveness(place, live);
            }
            StmtKind::Drop { .. } | StmtKind::Nop => {}
            StmtKind::StorageLive(local) | StmtKind::StorageDead(local) => {
                for slot in &self.local_regions[local.0 as usize] {
                    live.remove(&slot.region);
                }
            }
        }
    }

    fn terminator_liveness(
        &self,
        terminator: &Terminator,
        live: &mut HashSet<RegionVid>,
        call_contract: &dyn Fn(&FuncRef) -> CallRegionContract,
    ) {
        match terminator {
            Terminator::SwitchInt { discr, .. } => self.operand_liveness(discr, live),
            Terminator::Call { func, args, dest, .. } => {
                self.write_place_liveness(dest, live);
                match call_contract(func).access {
                    CallAccessContract::All => {
                        for arg in args {
                            self.operand_liveness(arg, live);
                        }
                    }
                    CallAccessContract::Fields(accesses) => {
                        for access in accesses {
                            let Some(argument) = args.get(access.argument) else { continue };
                            live.extend(self.operand_regions_at(argument, &access.projection));
                        }
                    }
                }
            }
            Terminator::Assert { cond, msg, .. } => {
                self.operand_liveness(cond, live);
                if let ember_mir::AssertKind::RefCellBorrow { file, line } = msg {
                    self.operand_liveness(file, live);
                    self.operand_liveness(line, live);
                }
            }
            Terminator::Return => {
                for slot in &self.local_regions[ember_mir::RETURN_LOCAL.0 as usize] {
                    live.insert(slot.region);
                }
            }
            Terminator::Goto(_) | Terminator::Unreachable => {}
        }
    }

    fn rvalue_liveness(&self, point: Point, rvalue: &Rvalue, live: &mut HashSet<RegionVid>) {
        match rvalue {
            Rvalue::Use(operand) | Rvalue::UnaryOp { operand, .. } => {
                self.operand_liveness(operand, live)
            }
            Rvalue::Cast { operand, .. } => self.operand_liveness(operand, live),
            Rvalue::BinaryOp { lhs, rhs, .. } => {
                self.operand_liveness(lhs, live);
                self.operand_liveness(rhs, live);
            }
            Rvalue::Aggregate { operands, .. } => {
                for operand in operands {
                    self.operand_liveness(operand, live);
                }
            }
            Rvalue::Repeat { value, .. } => self.operand_liveness(value, live),
            // Inspecting an enum tag selects a control-flow edge, but does not
            // read any payload. Retaining every payload slot here would turn
            // a later field projection through `Some`/`Ok` into a whole-value
            // region use and violate `[LT-20]`/`[LT-24]`.
            Rvalue::Discriminant(_) => {}
            Rvalue::Ref { place, .. } => {
                if let Some(paths) = self.capture_borrow_paths(point) {
                    for path in paths {
                        self.read_place_liveness(&project_place(place, path), live);
                    }
                } else {
                    self.read_place_liveness(place, live);
                }
            }
        }
    }

    fn operand_liveness(&self, operand: &Operand, live: &mut HashSet<RegionVid>) {
        if let Operand::Copy(place) | Operand::Move(place) = operand {
            self.read_place_liveness(place, live);
        }
    }

    fn read_place_liveness(&self, place: &Place, live: &mut HashSet<RegionVid>) {
        live.extend(self.place_regions(place));
    }

    fn write_place_liveness(&self, place: &Place, live: &mut HashSet<RegionVid>) {
        let slots = &self.local_regions[place.local.0 as usize];
        if slots.len() == 1 {
            let slot = &slots[0];
            if path_is_prefix(&place.projection, &slot.projection) {
                live.remove(&slot.region);
            } else {
                // Any projected mutation of a legacy one-region view requires
                // that view to remain valid, even when the projection is a
                // scalar metadata field beside its borrowed anchor.
                live.insert(slot.region);
            }
            return;
        }
        for slot in slots {
            if path_is_prefix(&place.projection, &slot.projection) {
                // Replacing a whole view or one of its containing fields kills
                // that slot's previous value.
                live.remove(&slot.region);
            } else if path_is_prefix(&slot.projection, &place.projection) {
                // A projection beyond a view leaf writes through the view and
                // therefore reads/needs its region rather than replacing it.
                live.insert(slot.region);
            }
        }
    }

    /// The reference a borrow goes *through*, for `[BRW-6]`'s reborrow.
    fn deref_base(&self, place: &Place) -> Option<RegionVid> {
        let mut candidates = self.local_regions[place.local.0 as usize]
            .iter()
            .filter(|slot| {
                path_is_prefix(&slot.projection, &place.projection)
                    && place.projection[slot.projection.len()..].iter().any(|projection| {
                        matches!(
                            projection,
                            Projection::Deref | Projection::Index(_) | Projection::ConstIndex(_)
                        )
                    })
            });
        let region = candidates.next()?.region;
        candidates.next().is_none().then_some(region)
    }
}

fn view_region_paths(types: &TypeTable, ty: Ty) -> Vec<Vec<Projection>> {
    match types.kind(ty) {
        // A reference is itself a view, but it must not collapse the field
        // slots of a multi-region value behind it. This is especially
        // load-bearing for `[LT-42]`: a closure environment stores `ref Pair`,
        // and a closure body that projects `pair.left` must retain only that
        // source field, not every field of `Pair`. An ordinary `ref T` still
        // has one slot when `T` contains no view fields.
        TyKind::Ref { inner, .. } => {
            let paths = view_region_paths(types, *inner);
            if paths.is_empty() {
                vec![Vec::new()]
            } else {
                paths
                    .into_iter()
                    .map(|mut path| {
                        path.insert(0, Projection::Deref);
                        path
                    })
                    .collect()
            }
        }
        TyKind::Str | TyKind::Span { .. } => vec![Vec::new()],
        TyKind::Struct(id) => types
            .struct_def(*id)
            .fields
            .iter()
            .enumerate()
            .flat_map(|(field, definition)| {
                view_region_paths(types, definition.ty).into_iter().map(move |mut path| {
                    path.insert(0, Projection::Field(field));
                    path
                })
            })
            .collect(),
        TyKind::Tuple(items) => items
            .iter()
            .enumerate()
            .flat_map(|(field, ty)| {
                view_region_paths(types, *ty).into_iter().map(move |mut path| {
                    path.insert(0, Projection::Field(field));
                    path
                })
            })
            .collect(),
        TyKind::Enum(id) => types
            .enum_def(*id)
            .variants
            .iter()
            .enumerate()
            .flat_map(|(variant, definition)| {
                definition.fields.iter().enumerate().flat_map(move |(field, definition)| {
                    view_region_paths(types, definition.ty).into_iter().map(move |mut path| {
                        path.insert(0, Projection::Field(field));
                        path.insert(0, Projection::Downcast(variant));
                        path
                    })
                })
            })
            .collect(),
        // A fixed array can contain arbitrarily many view values. Keep one
        // conservative slot for the array until element/range-sensitive
        // region vectors are implemented; never allocate metadata linear in
        // the source-level array length merely to represent type structure.
        TyKind::Array { elem, .. } if types.is_view(*elem) => vec![Vec::new()],
        _ => Vec::new(),
    }
}

fn project_place(place: &Place, path: &[Projection]) -> Place {
    let mut projected = place.clone();
    projected.projection.extend_from_slice(path);
    projected
}

fn path_is_prefix(prefix: &[Projection], path: &[Projection]) -> bool {
    prefix.len() <= path.len() && prefix.iter().zip(path).all(|(left, right)| left == right)
}

fn paths_overlap(left: &[Projection], right: &[Projection]) -> bool {
    path_is_prefix(left, right) || path_is_prefix(right, left)
}

fn aggregate_field_projection(kind: AggregateKind, field: usize) -> Vec<Projection> {
    match kind {
        AggregateKind::Struct(_) | AggregateKind::Tuple => vec![Projection::Field(field)],
        AggregateKind::Array => vec![Projection::ConstIndex(field as u64)],
        AggregateKind::Enum(_, variant) => {
            vec![Projection::Downcast(variant), Projection::Field(field)]
        }
    }
}

fn region_successors(terminator: &Terminator) -> Vec<BasicBlockId> {
    match terminator {
        Terminator::Goto(block) => vec![*block],
        Terminator::SwitchInt { targets, otherwise, .. } => {
            let mut blocks: Vec<BasicBlockId> = targets.iter().map(|(_, block)| *block).collect();
            blocks.push(*otherwise);
            blocks
        }
        Terminator::Call { next, .. } | Terminator::Assert { next, .. } => vec![*next],
        Terminator::Return | Terminator::Unreachable => Vec::new(),
    }
}

/// `[LT-4a]` names the growing `Arena` specifically as the one non-view
/// provenance source accepted by `@borrows`.
fn is_growing_arena(types: &TypeTable, ty: ember_types::Ty) -> bool {
    matches!(types.kind(ty), ember_types::TyKind::Struct(id)
        if types.struct_def(*id).name.as_str() == "Arena")
}

/// `[LT-1]` — which of a callee's arguments its returned view may point into.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Elision {
    /// The callee returns no view, so its result borrows nothing.
    Nothing,
    /// Rule 1 or 3: every view-typed parameter. The caller treats the result
    /// as borrowing all of them, which is the permissive reading `[LT-1]`
    /// chose over Rust's elision failure.
    Everything,
    /// `[LT-1a]` — `@borrows` named these parameter positions, and the caller
    /// may keep using the rest.
    Named(Vec<usize>),
}

impl Elision {
    pub(crate) fn ties(&self, argument: usize) -> bool {
        match self {
            Elision::Nothing => false,
            Elision::Everything => true,
            Elision::Named(indices) => indices.contains(&argument),
        }
    }
}
