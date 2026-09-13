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
//! One region variable per reference-typed local (§4.7 step 1) and one per
//! borrow expression. Every edge in the graph is an assignment, recorded in
//! the direction the *data* travels: `Flow { from, to }` for `to = from`. The
//! two things the borrow checker wants are the two directions of that one
//! graph:
//!
//! * **points** flow backwards along it — `points(from) ⊇ points(to)` — and
//!   answer "is this loan still live here";
//! * **provenance** flows forwards — `origins(to) ⊇ origins(from)` — and
//!   answers "which parameter did this reference come from", which is what
//!   `[LT-1]`'s elision and `[LT-1a]`'s `@borrows` are checked against.
//!
//! ## What it does not do yet
//!
//! Constraints are not location-sensitive: an edge propagates a whole point
//! set rather than the points reachable from where the assignment sits. §4.7
//! settles that deliberately — "Polonius-style location-sensitive reasoning is
//! not required for v1" — and the imprecision it costs needs a reference local
//! to be *re-seated*, which Ember has no syntax for: `r = ref mut m` writes
//! through `r` (`[TYP-14]`), it does not point `r` somewhere new. Every
//! reference local is assigned exactly once, at its declaration.

use std::collections::{HashMap, HashSet};

use ember_mir::{
    AggregateKind, BasicBlockId, Body, FuncRef, LocalId, LocalKind, Operand, Place, Projection,
    Rvalue, StmtKind, Terminator,
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

/// One input region named by an inferred multi-region result slot.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ResultRegionSource {
    /// A particular borrowed field of a view-typed argument.
    View { argument: usize, projection: Vec<Projection> },
    /// `[LT-4a]`'s narrow non-view Arena provenance source.
    Arena { argument: usize },
}

/// `[LT-22]` — the input regions from which one returned borrowed field is
/// derived. Both paths are relative to their public function types, not MIR
/// locals, so the record can be applied at any direct call site.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ResultFieldProvenance {
    pub result_projection: Vec<Projection>,
    pub sources: Vec<ResultRegionSource>,
}

/// An exact inferred field-to-source relation for a multi-region result.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ResultProvenanceSummary {
    pub fields: Vec<ResultFieldProvenance>,
}

/// The operations a callable may perform through a borrowed field.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum RegionAccessKind {
    Read,
    Write,
    BorrowShared,
    BorrowMut,
    Move,
    Return,
    Publish,
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

/// The exact operations a direct callable may perform through one parameter
/// field. The projection is relative to the parameter's public type.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ParameterFieldAccess {
    pub argument: usize,
    pub projection: Vec<Projection>,
    pub operations: Vec<RegionAccessKind>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum CallAccessContract {
    /// Opaque/unresolved dispatch: every field and operation is possible.
    All,
    /// Verified direct-call accesses. An empty vector means no view field is
    /// accessed and is distinct from unknown.
    Fields(Vec<ParameterFieldAccess>),
}

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
}

/// One edge of the constraint graph, in the direction the data travels.
#[derive(Copy, Clone, Debug)]
struct Flow {
    from: RegionVid,
    to: RegionVid,
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
        for (point, live_slots) in regions.slot_liveness(body, call_contract) {
            for region in live_slots {
                regions.points[region].insert(point);
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

        regions.collect_accesses(body, call_contract);
        let flows = regions.collect_flows(body, types, call_contract);
        regions.close(&flows);
        regions
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
    pub fn is_static_operand(&self, operand: &Operand) -> bool {
        match operand {
            Operand::Const(_) => true,
            Operand::Copy(place) | Operand::Move(place) => {
                let regions = self.place_regions(place);
                !regions.is_empty()
                    && regions.iter().all(|region| self.origins[*region].is_empty())
            }
        }
    }

    /// `[LT-35]` — seed the field operations actually present in this MIR.
    /// Assignment flows later carry these facts back to the parameter slots
    /// from which an accessed local or returned field was derived.
    fn collect_accesses(
        &mut self,
        body: &Body,
        call_contract: &dyn Fn(&FuncRef) -> CallRegionContract,
    ) {
        for block in &body.blocks {
            for stmt in &block.stmts {
                match &stmt.kind {
                    StmtKind::Assign { place, rvalue } => {
                        self.rvalue_accesses(rvalue);
                        // Replacing a view value changes local provenance but
                        // does not access its referent. A projection beyond a
                        // view leaf writes through it and is a real operation.
                        if self.assigned_place_regions(place).is_empty() {
                            self.mark_place_access(place, RegionAccessKind::Write);
                        }
                    }
                    StmtKind::CheckedBinaryOp { lhs, rhs, .. } => {
                        self.mark_operand_access(lhs);
                        self.mark_operand_access(rhs);
                    }
                    StmtKind::Drop { place, .. }
                        if self.drop_requires_regions[place.local.0 as usize] =>
                    {
                        self.mark_place_access(place, RegionAccessKind::Read);
                    }
                    StmtKind::StorageLive(_)
                    | StmtKind::StorageDead(_)
                    | StmtKind::Drop { .. }
                    | StmtKind::Nop => {}
                }
            }

            match &block.terminator {
                Terminator::SwitchInt { discr, .. } => self.mark_operand_access(discr),
                Terminator::Call { func, args, .. } => match call_contract(func).access {
                    CallAccessContract::All => {
                        for argument in args {
                            for region in self.operand_regions(argument) {
                                self.accesses[region].extend(ALL_REGION_ACCESSES);
                            }
                        }
                    }
                    CallAccessContract::Fields(accesses) => {
                        for access in accesses {
                            let Some(argument) = args.get(access.argument) else { continue };
                            for region in
                                self.operand_regions_at(argument, &access.projection)
                            {
                                self.accesses[region].extend(access.operations.iter().copied());
                            }
                        }
                    }
                },
                Terminator::Assert { cond, msg, .. } => {
                    self.mark_operand_access(cond);
                    if let ember_mir::AssertKind::RefCellBorrow { file, line } = msg {
                        self.mark_operand_access(file);
                        self.mark_operand_access(line);
                    }
                }
                Terminator::Return => {
                    for slot in &self.local_regions[ember_mir::RETURN_LOCAL.0 as usize] {
                        self.accesses[slot.region].insert(RegionAccessKind::Return);
                    }
                }
                Terminator::Goto(_) | Terminator::Unreachable => {}
            }
        }
    }

    fn rvalue_accesses(&mut self, rvalue: &Rvalue) {
        match rvalue {
            Rvalue::Use(operand) | Rvalue::UnaryOp { operand, .. } => {
                self.mark_operand_access(operand)
            }
            Rvalue::Cast { operand, .. } => self.mark_operand_access(operand),
            Rvalue::BinaryOp { lhs, rhs, .. } => {
                self.mark_operand_access(lhs);
                self.mark_operand_access(rhs);
            }
            Rvalue::Aggregate { operands, .. } => {
                for operand in operands {
                    self.mark_operand_access(operand);
                }
            }
            Rvalue::Repeat { value, .. } => self.mark_operand_access(value),
            Rvalue::Discriminant(place) => {
                self.mark_place_access(place, RegionAccessKind::Read)
            }
            Rvalue::Ref { place, mutable } => self.mark_place_access(
                place,
                if *mutable {
                    RegionAccessKind::BorrowMut
                } else {
                    RegionAccessKind::BorrowShared
                },
            ),
        }
    }

    fn mark_operand_access(&mut self, operand: &Operand) {
        match operand {
            Operand::Copy(place) => self.mark_place_access(place, RegionAccessKind::Read),
            Operand::Move(place) => self.mark_place_access(place, RegionAccessKind::Move),
            Operand::Const(_) => {}
        }
    }

    fn mark_place_access(&mut self, place: &Place, access: RegionAccessKind) {
        for region in self.place_regions(place) {
            self.accesses[region].insert(access);
        }
    }

    /// §4.7 step 2 — one walk of the body, collecting every assignment that
    /// moves a reference from one place to another.
    fn collect_flows(
        &mut self,
        body: &Body,
        types: &TypeTable,
        call_contract: &dyn Fn(&FuncRef) -> CallRegionContract,
    ) -> Vec<Flow> {
        let mut flows = Vec::new();
        let mut seeds: Vec<(RegionVid, Origin)> = Vec::new();
        for (block_index, block) in body.blocks.iter().enumerate() {
            for (index, stmt) in block.stmts.iter().enumerate() {
                let StmtKind::Assign { place, rvalue } = &stmt.kind else { continue };
                let destinations = self.assigned_place_regions(place);
                if destinations.is_empty() {
                    continue;
                }
                match rvalue {
                    Rvalue::Ref { place: borrowed, .. } => {
                        let point = Point { block: block_index, index };
                        if let Some(loan) = self.loan_region.get(&point).copied() {
                            for to in &destinations {
                                flows.push(Flow { from: loan, to: *to });
                            }
                            match self.deref_base(borrowed) {
                                // `[BRW-6]` — a reborrow is derived from the
                                // reference it goes through, so the base's
                                // region has to cover everywhere the new one
                                // reaches, and its provenance is the base's.
                                Some(base) => flows.push(Flow { from: base, to: loan }),
                                // A borrow names its own origin.
                                None => {
                                    let root = borrowed.local;
                                    seeds.push((
                                        loan,
                                        match body.local(root).kind {
                                            LocalKind::Arg => Origin::Param(root),
                                            _ => Origin::Local(root),
                                        },
                                    ));
                                }
                            }
                        }
                    }
                    Rvalue::Use(operand) | Rvalue::Cast { operand, .. } => {
                        connect_regions(
                            &mut flows,
                            &self.operand_regions(operand),
                            &destinations,
                        );
                    }
                    // `[LT-14]`, `[LT-16]`, `[LT-19]` — each aggregate field
                    // receives only the source region vector of its matching
                    // operand. The old all-operands-to-one-destination edge
                    // was exactly the legacy intersection model.
                    Rvalue::Aggregate { kind, operands } => {
                        if matches!(kind, AggregateKind::Array) {
                            // Fixed arrays of views deliberately have one
                            // conservative slot: allocating a region vector
                            // proportional to the source-level array length
                            // would violate the bounded metadata model. Every
                            // element must therefore feed that slot. Routing
                            // through `ConstIndex` would select no destination
                            // and silently lose provenance.
                            for operand in operands {
                                connect_all(
                                    &mut flows,
                                    &self.operand_regions(operand),
                                    &destinations,
                                );
                            }
                        } else {
                            for (field, operand) in operands.iter().enumerate() {
                                let mut projection = place.projection.clone();
                                projection.extend(aggregate_field_projection(*kind, field));
                                let field_place = Place { local: place.local, projection };
                                connect_regions(
                                    &mut flows,
                                    &self.operand_regions(operand),
                                    &self.assigned_place_regions(&field_place),
                                );
                            }
                        }
                    }
                    Rvalue::Repeat { value, .. } => {
                        connect_all(&mut flows, &self.operand_regions(value), &destinations);
                    }
                    Rvalue::BinaryOp { .. }
                    | Rvalue::UnaryOp { .. }
                    | Rvalue::Discriminant(_) => {}
                }
            }

            // `[LT-1]` at the call site: the result of a call may point into
            // any view-typed argument the callee's elision ties it to, so the
            // caller has to treat those as borrowed for as long as it holds
            // the result.
            if let Terminator::Call { func, args, dest, .. } = &block.terminator {
                let destinations = self.assigned_place_regions(dest);
                if !destinations.is_empty() {
                    match call_contract(func).result {
                        CallResultContract::Fields(summary) => {
                            for field in summary.fields {
                                let mut result = dest.clone();
                                result.projection.extend(field.result_projection);
                                let field_destinations = self.assigned_place_regions(&result);
                                for source in field.sources {
                                    match source {
                                        ResultRegionSource::View { argument, projection } => {
                                            let Some(arg) = args.get(argument) else { continue };
                                            connect_all(
                                                &mut flows,
                                                &self.operand_regions_at(arg, &projection),
                                                &field_destinations,
                                            );
                                        }
                                        ResultRegionSource::Arena { argument } => {
                                            let Some(Operand::Copy(place) | Operand::Move(place)) =
                                                args.get(argument)
                                            else {
                                                continue;
                                            };
                                            for to in &field_destinations {
                                                seeds.push((
                                                    *to,
                                                    match body.local(place.local).kind {
                                                        LocalKind::Arg => Origin::Param(place.local),
                                                        _ => Origin::Local(place.local),
                                                    },
                                                ));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        CallResultContract::Legacy(tied) => {
                            if destinations.len() > 1 {
                                for destination in &destinations {
                                    self.imprecise_provenance[*destination] = true;
                                }
                            }
                            for (index, arg) in args.iter().enumerate() {
                                if !tied.ties(index) {
                                    continue;
                                }
                                let sources = self.operand_regions(arg);
                                if !sources.is_empty() {
                                    // `[LT-35]` — without a field-sensitive
                                    // callable summary a call conservatively
                                    // requires every source and result slot.
                                    connect_all(&mut flows, &sources, &destinations);
                                } else if let Operand::Copy(place) | Operand::Move(place) = arg
                                    && is_growing_arena(types, body.local(place.local).ty)
                                {
                                    // `[LT-1a]`, `[LT-4a]` — Arena is the one
                                    // non-view parameter that may source
                                    // returned provenance.
                                    for to in &destinations {
                                        seeds.push((
                                            *to,
                                            match body.local(place.local).kind {
                                                LocalKind::Arg => Origin::Param(place.local),
                                                _ => Origin::Local(place.local),
                                            },
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        for (vid, origin) in seeds {
            self.origins[vid].insert(origin);
        }
        flows
    }

    /// The fixpoint. Points travel backwards along each edge and origins
    /// forwards; the graph is the same one.
    fn close(&mut self, flows: &[Flow]) {
        let mut changed = true;
        while changed {
            changed = false;
            for flow in flows {
                let carried: Vec<Point> = self.points[flow.to]
                    .iter()
                    .filter(|p| !self.points[flow.from].contains(*p))
                    .copied()
                    .collect();
                if !carried.is_empty() {
                    self.points[flow.from].extend(carried);
                    changed = true;
                }
                let holders: Vec<LocalId> = self.holders[flow.to]
                    .iter()
                    .filter(|l| !self.holders[flow.from].contains(*l))
                    .copied()
                    .collect();
                if !holders.is_empty() {
                    self.holders[flow.from].extend(holders);
                    changed = true;
                }
                let slots: Vec<RegionVid> = self.carried_slots[flow.to]
                    .iter()
                    .filter(|slot| !self.carried_slots[flow.from].contains(*slot))
                    .copied()
                    .collect();
                if !slots.is_empty() {
                    self.carried_slots[flow.from].extend(slots);
                    changed = true;
                }
                if self.imprecise_provenance[flow.from]
                    && !self.imprecise_provenance[flow.to]
                {
                    self.imprecise_provenance[flow.to] = true;
                    changed = true;
                }
                let accesses: Vec<RegionAccessKind> = self.accesses[flow.to]
                    .iter()
                    .filter(|access| !self.accesses[flow.from].contains(*access))
                    .copied()
                    .collect();
                if !accesses.is_empty() {
                    self.accesses[flow.from].extend(accesses);
                    changed = true;
                }
                let origins: Vec<Origin> = self.origins[flow.from]
                    .iter()
                    .filter(|o| !self.origins[flow.to].contains(*o))
                    .copied()
                    .collect();
                if !origins.is_empty() {
                    self.origins[flow.to].extend(origins);
                    changed = true;
                }
            }
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
                self.statement_liveness(&stmt.kind, &mut live);
                points.insert(Point { block: block_index, index }, live.clone());
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
        for stmt in block.stmts.iter().rev() {
            self.statement_liveness(&stmt.kind, &mut live);
        }
        Some(live)
    }

    fn statement_liveness(&self, kind: &StmtKind, live: &mut HashSet<RegionVid>) {
        match kind {
            StmtKind::Assign { place, rvalue } => {
                self.write_place_liveness(place, live);
                self.rvalue_liveness(rvalue, live);
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

    fn rvalue_liveness(&self, rvalue: &Rvalue, live: &mut HashSet<RegionVid>) {
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
            Rvalue::Discriminant(place) => self.read_place_liveness(place, live),
            Rvalue::Ref { place, .. } => self.read_place_liveness(place, live),
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
        if place.projection.iter().any(|p| matches!(p, Projection::Deref)) {
            let regions = self.place_regions(place);
            (regions.len() == 1).then(|| regions[0])
        } else {
            None
        }
    }
}

fn view_region_paths(types: &TypeTable, ty: Ty) -> Vec<Vec<Projection>> {
    match types.kind(ty) {
        TyKind::Ref { .. } | TyKind::Str | TyKind::Span { .. } => vec![Vec::new()],
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

fn connect_regions(flows: &mut Vec<Flow>, sources: &[RegionVid], destinations: &[RegionVid]) {
    if sources.len() == destinations.len() {
        flows.extend(
            sources
                .iter()
                .zip(destinations)
                .map(|(from, to)| Flow { from: *from, to: *to }),
        );
    } else {
        connect_all(flows, sources, destinations);
    }
}

fn connect_all(flows: &mut Vec<Flow>, sources: &[RegionVid], destinations: &[RegionVid]) {
    for from in sources {
        for to in destinations {
            flows.push(Flow { from: *from, to: *to });
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
