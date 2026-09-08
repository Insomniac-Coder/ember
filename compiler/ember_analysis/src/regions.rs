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
    Body, FuncRef, LocalId, LocalKind, Operand, Place, Projection, Rvalue, StmtKind, Terminator,
};
use ember_types::TypeTable;

/// A point in the CFG: a statement index within a block, where `stmts.len()`
/// is the terminator.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct Point {
    pub block: usize,
    pub index: usize,
}

/// A region variable. Locals take the low ids so that `local_region` is a
/// lookup rather than a map; borrow expressions take the rest.
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
    /// The region variable of each local, where it has one.
    local_region: Vec<Option<RegionVid>>,
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
    /// `live` is the backward liveness the borrow checker already computes;
    /// `elision` answers, for a call, which of the callee's view-typed
    /// arguments its result may borrow (`[LT-1]`).
    pub fn infer(
        body: &Body,
        types: &TypeTable,
        live: &HashMap<Point, HashSet<LocalId>>,
        elision: &dyn Fn(&FuncRef) -> Elision,
    ) -> Regions {
        let mut local_region = vec![None; body.locals.len()];
        let mut next = 0;
        for (index, decl) in body.locals.iter().enumerate() {
            if types.is_view(decl.ty) {
                local_region[index] = Some(next);
                next += 1;
            }
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

        let mut regions = Regions {
            points: vec![HashSet::new(); next],
            origins: vec![HashSet::new(); next],
            holders: vec![HashSet::new(); next],
            local_region,
            loan_region,
        };

        for (index, _) in body.locals.iter().enumerate() {
            if let Some(vid) = regions.local_region[index] {
                regions.holders[vid].insert(LocalId(index as u32));
            }
        }

        // Step 3, the seed: a region includes every point at which a local
        // whose type mentions it is live.
        for (point, locals) in live {
            for local in locals {
                if let Some(vid) = regions.local_region[local.0 as usize] {
                    regions.points[vid].insert(*point);
                }
            }
        }

        // A view-typed parameter is where provenance starts. `[LT-1]` is
        // stated over exactly these: what the caller owns, which the callee
        // may hand back and may not outlive.
        for (local, _) in body.args() {
            if let Some(vid) = regions.local_region[local.0 as usize] {
                regions.origins[vid].insert(Origin::Param(local));
            }
        }

        let flows = regions.collect_flows(body, elision);
        regions.close(&flows);
        regions
    }

    /// The region of the borrow expression at `point`, if there is one.
    pub fn loan_region(&self, point: Point) -> Option<RegionVid> {
        self.loan_region.get(&point).copied()
    }

    /// The region of a local, if it holds a view.
    pub fn local_region(&self, local: LocalId) -> Option<RegionVid> {
        self.local_region[local.0 as usize]
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

    /// §4.7 step 2 — one walk of the body, collecting every assignment that
    /// moves a reference from one place to another.
    fn collect_flows(&mut self, body: &Body, elision: &dyn Fn(&FuncRef) -> Elision) -> Vec<Flow> {
        let mut flows = Vec::new();
        let mut seeds: Vec<(RegionVid, Origin)> = Vec::new();
        for (block_index, block) in body.blocks.iter().enumerate() {
            for (index, stmt) in block.stmts.iter().enumerate() {
                let StmtKind::Assign { place, rvalue } = &stmt.kind else { continue };
                let Some(to) = self.place_region(place) else { continue };
                match rvalue {
                    Rvalue::Ref { place: borrowed, .. } => {
                        let point = Point { block: block_index, index };
                        if let Some(loan) = self.loan_region.get(&point).copied() {
                            flows.push(Flow { from: loan, to });
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
                        if let Some(from) = self.operand_region(operand) {
                            flows.push(Flow { from, to });
                        }
                    }
                    // `[LT-2]` — a view struct has one region, so every
                    // view-typed field it is built from constrains it.
                    Rvalue::Aggregate { operands, .. } => {
                        for operand in operands {
                            if let Some(from) = self.operand_region(operand) {
                                flows.push(Flow { from, to });
                            }
                        }
                    }
                    Rvalue::Repeat { value, .. } => {
                        if let Some(from) = self.operand_region(value) {
                            flows.push(Flow { from, to });
                        }
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
                if let Some(to) = self.place_region(dest) {
                    let tied = elision(func);
                    for (index, arg) in args.iter().enumerate() {
                        if !tied.ties(index) {
                            continue;
                        }
                        if let Some(from) = self.operand_region(arg) {
                            flows.push(Flow { from, to });
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

    /// The region a place denotes, or `None` when it denotes a referent
    /// rather than a reference. `(*r) = 7` writes *through* `r`; nothing
    /// about `r`'s own region changes.
    fn place_region(&self, place: &Place) -> Option<RegionVid> {
        if place.projection.iter().any(|p| matches!(p, Projection::Deref)) {
            return None;
        }
        self.local_region[place.local.0 as usize]
    }

    fn operand_region(&self, operand: &Operand) -> Option<RegionVid> {
        match operand {
            Operand::Copy(place) | Operand::Move(place) => self.place_region(place),
            // `[LT-3]` — a literal has the static region, which outlives
            // everything and constrains nothing.
            Operand::Const(_) => None,
        }
    }

    /// The reference a borrow goes *through*, for `[BRW-6]`'s reborrow.
    fn deref_base(&self, place: &Place) -> Option<RegionVid> {
        if place.projection.iter().any(|p| matches!(p, Projection::Deref)) {
            self.local_region[place.local.0 as usize]
        } else {
            None
        }
    }
}

/// `[LT-1]` — which of a callee's arguments its returned view may point into.
#[derive(Clone, Debug)]
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
    fn ties(&self, argument: usize) -> bool {
        match self {
            Elision::Nothing => false,
            Elision::Everything => true,
            Elision::Named(indices) => indices.contains(&argument),
        }
    }
}
