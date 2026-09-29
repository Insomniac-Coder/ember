//! `[OPT-2]`, `[OPT-3]` — loop versioning for bounds checks.
//!
//! A counted loop `for i in a..b` (or `a..=b`) whose body indexes a view at
//! `i + c` for constants `c`, with the view's base and length invariant in the
//! loop, gets one entry test that every index the loop can produce is in
//! bounds, an unchecked copy of the loop taken when it passes, and the
//! original checked loop otherwise. Behaviour is unchanged in every case: the
//! unchecked copy runs only when none of its removed checks could fail.
//!
//! Every counted loop is considered, outer loops first, so the loops inside an
//! outer loop's copies are versioned in each copy.
//!
//! **Invariance.** A view is invariant when nothing the loop writes can be the
//! view or a place on the path to it. What the loop writes is read through
//! everything it runs:
//! - its own statements;
//! - every call, through a summary of what the callee (and everything it
//!   calls) can write that its caller can see: class fields by class family,
//!   what it writes through each reference parameter, and whether it may write
//!   anywhere (a raw pointer, a reference of unknown origin, foreign code). A
//!   virtual call reads the summaries of every override in the family; an
//!   interface call, a call through a callable value or a built-in that runs
//!   program code reads the union of every function in the program;
//! - every drop, through the `drop` functions the dropped type can run.
//!
//! Memory reached through a reference parameter and memory reached through a
//! class handle are never the same place as far as the loop can observe: a
//! caller that lends a class field as a `mut` argument holds a write access to
//! it for the call (`[HEAP-5]`, `[EXC-1]`), so reaching that field through a
//! handle panics before anything is changed. A `Sync` object may be written by
//! another thread and is never treated as invariant.
//!
//! An index is the counter plus a constant, through copies, adds, subtracts
//! and the final conversion to `usize`, each defined once in the loop in a
//! block that dominates the check.
//!
//! The original blocks stay the checked loop, so every side table that names
//! them stays valid; the unchecked copy and the entry test are new blocks.

use std::collections::{HashMap, HashSet};

use ember_mir::{
    AssertKind, BasicBlock, BasicBlockId, BinOp, Body, Builtin, CastKind, CheckKind, CheckProof, Const, FuncRef, LocalDecl,
    LocalId, LocalKind, Operand, Place, Projection, RemovedCheck, Rvalue, Stmt, StmtKind, Terminator, UnOp,
};
use ember_types::{ClassId, CommonTypes, Ty, TyKind, TypeTable};

use crate::access::class_family;
use crate::range_facts::Analysis;
use crate::borrows::quiet_builtin;
use crate::regions::place_type;

/// Version every qualifying loop in `bodies` (`[OPT-2]`), then group the
/// overflow checks of every loop in vectorisable form (`[SIMD-7]`); returns
/// how many loops were versioned and how many grouped.
pub fn version_bounds_checked_loops_all(bodies: &mut [Body], types: &TypeTable, common: &CommonTypes) -> (usize, usize) {
    let summaries = Summaries::compute(bodies, types);
    let mut versioned = 0;
    let mut grouped = 0;
    for body in bodies.iter_mut() {
        versioned += version_loops(body, types, common, &summaries);
        grouped += group_overflow_checks(body, types, common);
    }
    (versioned, grouped)
}

fn version_loops(body: &mut Body, types: &TypeTable, common: &CommonTypes, summaries: &Summaries) -> usize {
    // Outer loops first: a loop's copies then carry the loops inside it, and
    // each of those is versioned in turn. A header that does not qualify
    // never starts to, so it is not looked at again.
    let mut settled: HashSet<usize> = HashSet::new();
    let mut count = 0;
    loop {
        let facts = BodyFacts::new(body, types);
        let ranges = Analysis::run(body, types, common);
        let mut best: Option<Plan> = None;
        for header in 0..body.blocks.len() {
            if settled.contains(&header) {
                continue;
            }
            match plan(body, types, summaries, &facts, ranges.as_ref(), header) {
                Some(plan) => {
                    if best.as_ref().map_or(true, |best| plan.region.len() > best.region.len()) {
                        best = Some(plan);
                    }
                }
                None => {
                    settled.insert(header);
                }
            }
        }
        let Some(plan) = best else { break };
        let header = plan.header;
        let fast = apply(body, types, plan);
        settled.insert(header);
        settled.insert(fast);
        count += 1;
    }
    count
}

/// One removable check: the assert's block, the largest index it can see,
/// and the view's length.
struct Check {
    block: usize,
    bound: IndexBound,
    len: Operand,
    /// The index's type, `usize`: what the entry test compares in.
    index_ty: Ty,
}

/// The largest index a check sees in the loop.
#[derive(Clone, Copy, PartialEq, Debug)]
enum IndexBound {
    /// `counter + offset` (`[OPT-2]`).
    Counter(i128),
    /// At most a constant (`[RNG-4]`: `i % 4`, a mask, an inner counter).
    Constant(i128),
    /// At most `local + c`, for a local the loop does not write (`[RNG-4]`).
    Local(LocalId, i128),
}

struct Plan {
    header: usize,
    region: Vec<usize>,
    counter: LocalId,
    limit: LocalId,
    inclusive: bool,
    checks: Vec<Check>,
}

/// Offsets beyond this are left checked, so the entry test's arithmetic has
/// room in every counter type of at least 32 bits.
const MAX_OFFSET: i128 = 1 << 20;

fn plan(
    body: &Body,
    types: &TypeTable,
    summaries: &Summaries,
    facts: &BodyFacts,
    ranges: Option<&Analysis>,
    header: usize,
) -> Option<Plan> {
    let head = body.blocks.get(header)?;
    let Terminator::SwitchInt { discr: Operand::Copy(discr), targets, otherwise } = &head.terminator else {
        return None;
    };
    if !discr.projection.is_empty() || targets.len() != 1 || targets[0].0 != 0 {
        return None;
    }
    let (op, counter, limit) = head.stmts.iter().rev().find_map(|stmt| match &stmt.kind {
        StmtKind::Assign {
            place,
            rvalue: Rvalue::BinaryOp { op: op @ (BinOp::Lt | BinOp::Le), lhs: Operand::Copy(lhs), rhs: Operand::Copy(rhs) },
        } if *place == Place::local(discr.local) && lhs.projection.is_empty() && rhs.projection.is_empty() => {
            Some((*op, lhs.local, rhs.local))
        }
        _ => None,
    })?;
    let counter_ty = body.local(counter).ty;
    let width = ember_types::bit_width(types, counter_ty)?;
    if !(32..=64).contains(&width) || body.local(limit).ty != counter_ty {
        return None;
    }

    // The loop: every block that reaches the header's one back edge without
    // passing through the header.
    let predecessors = predecessors(body);
    let entry = otherwise.0 as usize;
    let reach = reachable_from(body, entry, header);
    let back: Vec<usize> = predecessors[header].iter().copied().filter(|p| reach.contains(p)).collect();
    let [step] = back.as_slice() else { return None };
    let mut region: HashSet<usize> = HashSet::new();
    let mut work = vec![*step];
    while let Some(block) = work.pop() {
        if block == header || !region.insert(block) {
            continue;
        }
        work.extend(predecessors[block].iter().copied());
    }
    if !region.contains(&entry) {
        return None;
    }
    // One entry: the header.
    if region.iter().any(|&b| predecessors[b].iter().any(|p| *p != header && !region.contains(p))) {
        return None;
    }
    // The step is exactly `counter = counter + 1`.
    let step_block = &body.blocks[*step];
    let increments = step_block
        .stmts
        .iter()
        .filter(|stmt| matches!(&stmt.kind, StmtKind::Assign { place, .. } if place.local == counter))
        .count();
    let is_increment = step_block.stmts.iter().any(|stmt| {
        matches!(&stmt.kind,
            StmtKind::Assign { place, rvalue: Rvalue::BinaryOp { op: BinOp::Add, lhs: Operand::Copy(lhs), rhs: Operand::Const(Const::Int { value: 1, .. }) } }
                if *place == Place::local(counter) && *lhs == Place::local(counter))
    });
    if increments != 1 || !is_increment || !matches!(step_block.terminator, Terminator::Goto(target) if target.0 as usize == header) {
        return None;
    }

    let mut all: Vec<usize> = region.iter().copied().collect();
    all.push(header);
    all.sort_unstable();
    let written = loop_writes(body, types, summaries, facts, &all);
    // Nothing else in the loop writes the counter or the limit.
    let touches = |local: LocalId| Loc { root: Root::Local(local), path: Vec::new() };
    if written.iter().any(|(loc, block)| {
        may_alias(loc, &touches(limit), facts) || (*block != *step && may_alias(loc, &touches(counter), facts))
    }) {
        return None;
    }

    let definitions = definitions(body, &all);
    let dominators = dominators(body, header, &all);
    let mut checks = Vec::new();
    for &block in &region {
        let Terminator::Assert { expected: true, msg: AssertKind::Bounds { len, index: Operand::Copy(index) }, .. } =
            &body.blocks[block].terminator
        else {
            continue;
        };
        if !index.projection.is_empty() {
            continue;
        }
        let invariant = match len {
            Operand::Const(Const::Int { .. }) => true,
            Operand::Copy(len_place) => view_invariant(body, types, facts, len_place, &written),
            _ => false,
        };
        if !invariant {
            continue;
        }
        let index_ty = body.local(index.local).ty;
        let bound = match affine(body, types, &definitions, &dominators, counter, index.local, block, 0) {
            Some(offset) if offset.abs() <= MAX_OFFSET => IndexBound::Counter(offset),
            Some(_) => continue,
            None => {
                // `[RNG-4]` — what the range facts know of the index here,
                // which holds on every iteration.
                let Some((hi, symbolic)) = ranges.and_then(|r| r.upper_bounds(block, &Operand::Copy(index.clone())))
                else {
                    continue;
                };
                let index_max = ember_types::int_max(types, index_ty).and_then(|m| i128::try_from(m).ok());
                let unwritten = |local: LocalId| {
                    let loc = Loc { root: Root::Local(local), path: Vec::new() };
                    !written.iter().any(|(written, _)| may_alias(written, &loc, facts))
                };
                // The entry test computes in the local's type, which must
                // convert to the index's without loss.
                let usable = |local: LocalId| {
                    let ty = body.local(local).ty;
                    ember_types::bit_width(types, ty).is_some_and(|w| w <= 64)
                        && ember_types::int_max(types, ty) <= ember_types::int_max(types, index_ty)
                };
                if index_max.is_some_and(|max| hi < max) {
                    IndexBound::Constant(hi)
                } else if let Some((local, c)) = symbolic
                    .into_iter()
                    .find(|(local, c)| *local != counter && c.abs() <= MAX_OFFSET && unwritten(*local) && usable(*local))
                {
                    IndexBound::Local(local, c)
                } else {
                    continue;
                }
            }
        };
        checks.push(Check { block, bound, len: len.clone(), index_ty });
    }
    if checks.is_empty() {
        return None;
    }
    checks.sort_by_key(|check| check.block);
    let mut region: Vec<usize> = region.into_iter().collect();
    region.sort_unstable();
    Some(Plan { header, region, counter, limit, inclusive: op == BinOp::Le, checks })
}

/// The view whose length is `len_place` (`view.len`) keeps its base and
/// length for the whole loop: nothing the loop writes can be the view or a
/// place on the path to it.
fn view_invariant(
    body: &Body,
    types: &TypeTable,
    facts: &BodyFacts,
    len_place: &Place,
    written: &[(Loc, usize)],
) -> bool {
    let Some((Projection::Field(1), view)) = len_place.projection.split_last() else { return false };
    let view = Place { local: len_place.local, projection: view.to_vec() };
    if !matches!(types.kind(place_type(body, types, &view)), TyKind::Vec { .. } | TyKind::Span { .. }) {
        return false;
    }
    let mut prefixes = Vec::new();
    for k in 0..=view.projection.len() {
        let prefix = Place { local: view.local, projection: view.projection[..k].to_vec() };
        if k < view.projection.len() {
            match view.projection[k] {
                Projection::Field(_) | Projection::Deref | Projection::Downcast(_) => {}
                _ => return false,
            }
            // `[THR-1]` — a `Sync` object can be written by another thread.
            if let TyKind::Class(class) = types.kind(place_type(body, types, &prefix)) {
                if types.class_def(*class).is_sync {
                    return false;
                }
            }
        }
        let Some(loc) = locate(body, types, &facts.origins, &prefix, 0) else { return false };
        prefixes.push(loc);
    }
    !written.iter().any(|(loc, _)| prefixes.iter().any(|prefix| may_alias(loc, prefix, facts)))
}

// ---------------------------------------------------------------------------
// Where a place's memory is reached from.

/// One step inside a value: a field, or an enum variant's payload.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
enum Step {
    Field(usize),
    Variant(usize),
}

/// The root a place's memory is reached from.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
enum Root {
    /// A local of this function (not through a reference).
    Local(LocalId),
    /// The referent of reference parameter `n` (its local index).
    Param(u32),
    /// A class object, by the root of its class family: any handle of the
    /// family can name it.
    Heap(ClassId),
    /// Anything: a raw pointer or a reference of unknown origin.
    Anywhere,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
struct Loc {
    root: Root,
    path: Vec<Step>,
}

/// What one function knows about its own places.
struct BodyFacts {
    /// A local whose every assignment takes a reference to the same place.
    origins: HashMap<LocalId, Place>,
    /// Locals some mutable reference or pointer is taken of: an unknown write
    /// may reach them.
    escaped: HashSet<LocalId>,
}

impl BodyFacts {
    fn new(body: &Body, types: &TypeTable) -> BodyFacts {
        let mut assigned: HashMap<LocalId, Vec<Option<Place>>> = HashMap::new();
        let mut escaped = HashSet::new();
        for block in &body.blocks {
            for stmt in &block.stmts {
                match &stmt.kind {
                    StmtKind::Assign { place, rvalue } => {
                        if let Rvalue::Ref { place: target, mutable } = rvalue {
                            if *mutable || matches!(types.kind(place_type(body, types, place)), TyKind::Ptr { .. }) {
                                escaped.insert(target.local);
                            }
                        }
                        if place.projection.is_empty() {
                            let origin = match rvalue {
                                Rvalue::Ref { place: target, .. } => Some(target.clone()),
                                _ => None,
                            };
                            assigned.entry(place.local).or_default().push(origin);
                        }
                    }
                    StmtKind::CheckedBinaryOp { dest, .. } if dest.projection.is_empty() => {
                        assigned.entry(dest.local).or_default().push(None);
                    }
                    _ => {}
                }
            }
            if let Terminator::Call { dest, .. } = &block.terminator {
                if dest.projection.is_empty() {
                    assigned.entry(dest.local).or_default().push(None);
                }
            }
        }
        let origins = assigned
            .into_iter()
            .filter_map(|(local, origins)| {
                let first = origins.first()?.clone()?;
                origins.iter().all(|origin| origin.as_ref() == Some(&first)).then_some((local, first))
            })
            .collect();
        BodyFacts { origins, escaped }
    }
}

fn is_param(body: &Body, local: LocalId) -> bool {
    local.0 >= 1 && (local.0 as usize) <= body.arg_count
}

/// Where `place`'s memory is reached from; `None` for a place inside an
/// element of a buffer, which no view's base or length can be.
fn locate(
    body: &Body,
    types: &TypeTable,
    origins: &HashMap<LocalId, Place>,
    place: &Place,
    depth: usize,
) -> Option<Loc> {
    let mut loc = Loc { root: Root::Local(place.local), path: Vec::new() };
    let mut in_element = false;
    for (index, projection) in place.projection.iter().enumerate() {
        let prefix = Place { local: place.local, projection: place.projection[..index].to_vec() };
        let prefix_ty = place_type(body, types, &prefix);
        match projection {
            Projection::Deref => {
                let through_reference = matches!(types.kind(prefix_ty), TyKind::Ref { .. } | TyKind::Ptr { .. });
                loc = match (&loc.root, loc.path.is_empty() && !in_element, through_reference) {
                    (Root::Local(local), true, true) if is_param(body, *local) => {
                        Loc { root: Root::Param(local.0), path: Vec::new() }
                    }
                    (Root::Local(local), true, true) if depth < 8 => match origins.get(local) {
                        Some(origin) => locate(body, types, origins, origin, depth + 1)?,
                        None => Loc { root: Root::Anywhere, path: Vec::new() },
                    },
                    _ => Loc { root: Root::Anywhere, path: Vec::new() },
                };
                in_element = false;
            }
            Projection::Field(field) => {
                if let TyKind::Class(class) = types.kind(prefix_ty) {
                    loc = Loc { root: Root::Heap(class_family(types, *class)), path: vec![Step::Field(*field)] };
                    in_element = false;
                } else {
                    loc.path.push(Step::Field(*field));
                }
            }
            Projection::Downcast(variant) => loc.path.push(Step::Variant(*variant)),
            Projection::Index(_) | Projection::ConstIndex(_) => in_element = true,
            Projection::Column(_) => {
                loc = Loc { root: Root::Anywhere, path: Vec::new() };
                in_element = false;
            }
        }
    }
    (!in_element).then_some(loc)
}

/// Whether a write to `written` can change the memory at `place`.
fn may_alias(written: &Loc, place: &Loc, facts: &BodyFacts) -> bool {
    let related = |a: &[Step], b: &[Step]| {
        let n = a.len().min(b.len());
        a[..n] == b[..n]
    };
    match (&written.root, &place.root) {
        (Root::Anywhere, Root::Local(local)) | (Root::Local(local), Root::Anywhere) => facts.escaped.contains(local),
        (Root::Anywhere, _) | (_, Root::Anywhere) => true,
        (Root::Local(a), Root::Local(b)) => a == b && related(&written.path, &place.path),
        (Root::Param(a), Root::Param(b)) => a == b && related(&written.path, &place.path),
        (Root::Heap(a), Root::Heap(b)) => a == b && related(&written.path, &place.path),
        _ => false,
    }
}

// ---------------------------------------------------------------------------
// What a loop writes.

/// Everything the loop's blocks can write, with the block that writes it.
fn loop_writes(
    body: &Body,
    types: &TypeTable,
    summaries: &Summaries,
    facts: &BodyFacts,
    blocks: &[usize],
) -> Vec<(Loc, usize)> {
    let mut written = Vec::new();
    for &block in blocks {
        let mut out = Vec::new();
        block_writes(body, types, summaries, &facts.origins, &body.blocks[block], &mut out);
        written.extend(out.into_iter().map(|loc| (loc, block)));
    }
    written
}

/// The places one block can write: its statements, its drops (through the
/// `drop` functions they can run) and its call.
fn block_writes(
    body: &Body,
    types: &TypeTable,
    summaries: &Summaries,
    origins: &HashMap<LocalId, Place>,
    block: &BasicBlock,
    out: &mut Vec<Loc>,
) {
    let place_written = |place: &Place, out: &mut Vec<Loc>| {
        if let Some(loc) = locate(body, types, origins, place, 0) {
            out.push(loc);
        }
    };
    for stmt in &block.stmts {
        match &stmt.kind {
            StmtKind::Assign { place, .. } => place_written(place, out),
            StmtKind::CheckedBinaryOp { dest, overflow, .. } => {
                place_written(dest, out);
                place_written(overflow, out);
            }
            StmtKind::Drop { place, .. } => {
                place_written(place, out);
                summaries.drop_writes(types, place_type(body, types, place)).locs(out);
            }
            StmtKind::StorageLive(local) | StmtKind::StorageDead(local) => {
                out.push(Loc { root: Root::Local(*local), path: Vec::new() });
            }
            StmtKind::BeginAccess { .. }
            | StmtKind::BeginAccessTransfer { .. }
            | StmtKind::EndAccess { .. }
            | StmtKind::EndAccessTransfer { .. }
            | StmtKind::Nop => {}
        }
    }
    if let Terminator::Call { func, args, dest, .. } = &block.terminator {
        place_written(dest, out);
        // The referent of an argument: what the callee reaches through it.
        let referent = |operand: &Operand| -> Option<Option<Loc>> {
            let (Operand::Copy(place) | Operand::Move(place)) = operand else { return None };
            if !matches!(types.kind(place_type(body, types, place)), TyKind::Ref { .. } | TyKind::Ptr { .. }) {
                return None;
            }
            let mut deref = place.clone();
            deref.projection.push(Projection::Deref);
            Some(locate(body, types, origins, &deref, 0))
        };
        let mutable_referents = |out: &mut Vec<Loc>| {
            for arg in args {
                let (Operand::Copy(place) | Operand::Move(place)) = arg else { continue };
                if matches!(
                    types.kind(place_type(body, types, place)),
                    TyKind::Ref { mutable: true, .. } | TyKind::Ptr { mutable: true, .. }
                ) {
                    if let Some(Some(loc)) = referent(arg) {
                        out.push(loc);
                    }
                }
            }
        };
        let through = |writes: &Writes, out: &mut Vec<Loc>| {
            writes.locs(out);
            for (param, path) in &writes.params {
                let Some(arg) = (*param as usize).checked_sub(1).and_then(|index| args.get(index)) else {
                    out.push(Loc { root: Root::Anywhere, path: Vec::new() });
                    continue;
                };
                match referent(arg) {
                    Some(Some(mut loc)) => {
                        loc.path.extend(path.iter().cloned());
                        loc.path.truncate(MAX_PATH);
                        out.push(loc);
                    }
                    Some(None) => {}
                    None => out.push(Loc { root: Root::Anywhere, path: Vec::new() }),
                }
            }
        };
        match func {
            FuncRef::Direct { symbol, .. } => match summaries.of(symbol) {
                Some(writes) => through(writes, out),
                None => {
                    out.push(Loc { root: Root::Anywhere, path: Vec::new() });
                    mutable_referents(out);
                }
            },
            FuncRef::Virtual { owner, slot, .. } => match summaries.overrides(types, *owner, *slot) {
                Some(writes) => through(&writes, out),
                None => {
                    summaries.everything.locs(out);
                    mutable_referents(out);
                }
            },
            // Making a view of a list, reborrowing one, or reading a length
            // writes nothing through its argument; what is later written
            // through the view is an element, which no header is.
            FuncRef::Builtin {
                which:
                    Builtin::SpanFrom { .. }
                    | Builtin::SpanReborrow
                    | Builtin::SpanSharedReborrow
                    | Builtin::SpanLen
                    | Builtin::ArrayLen
                    | Builtin::StringLen,
                ..
            } => {}
            FuncRef::Builtin { which, arg_ty } if quiet_builtin(types, which, *arg_ty) => mutable_referents(out),
            FuncRef::DynBoxNew { .. } => mutable_referents(out),
            FuncRef::Builtin { .. } | FuncRef::Interface { .. } | FuncRef::Indirect { .. } => {
                summaries.everything.locs(out);
                mutable_referents(out);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// What each function can write that its caller can see.

/// A function's writes its caller can observe, read through every call it
/// makes and every drop it runs.
#[derive(Clone, Default, PartialEq, Debug)]
struct Writes {
    /// Class fields, by class family and path.
    heap: HashSet<(ClassId, Vec<Step>)>,
    /// Through reference parameter `n` (its local index), at a path.
    params: HashSet<(u32, Vec<Step>)>,
    /// It may write anywhere.
    anywhere: bool,
}

/// Paths are kept to this many steps. A shorter path is a prefix of every
/// longer one, so it is related to more places: cutting one is safe.
const MAX_PATH: usize = 4;

/// Summary rounds before every function is taken to write anywhere.
const MAX_ROUNDS: usize = 64;

impl Writes {
    fn add(&mut self, mut loc: Loc) {
        loc.path.truncate(MAX_PATH);
        match loc.root {
            Root::Local(_) => {}
            Root::Param(param) => {
                self.params.insert((param, loc.path));
            }
            Root::Heap(family) => {
                self.heap.insert((family, loc.path));
            }
            Root::Anywhere => self.anywhere = true,
        }
    }

    fn join(&mut self, other: &Writes) {
        self.heap.extend(other.heap.iter().cloned());
        self.params.extend(other.params.iter().cloned());
        self.anywhere |= other.anywhere;
    }

    /// The writes a caller sees, other than through its own arguments.
    fn locs(&self, out: &mut Vec<Loc>) {
        for (family, path) in &self.heap {
            out.push(Loc { root: Root::Heap(*family), path: path.clone() });
        }
        if self.anywhere {
            out.push(Loc { root: Root::Anywhere, path: Vec::new() });
        }
    }
}

struct Summaries {
    by_symbol: HashMap<String, Writes>,
    /// `(class, virtual slot)` of each method body, for a virtual call.
    methods: Vec<(ClassId, usize, String)>,
    /// The union of every function: a call whose target is not known.
    everything: Writes,
}

impl Summaries {
    fn compute(bodies: &[Body], types: &TypeTable) -> Summaries {
        let mut summaries = Summaries {
            by_symbol: bodies
                .iter()
                .filter(|body| !body.is_extern_declaration && !body.is_abstract)
                .map(|body| (body.symbol.clone(), Writes::default()))
                .collect(),
            methods: bodies
                .iter()
                .filter(|body| !body.is_abstract)
                .filter_map(|body| Some((body.class_owner?, body.class_virtual_slot?, body.symbol.clone())))
                .collect(),
            everything: Writes::default(),
        };
        let facts: Vec<BodyFacts> = bodies.iter().map(|body| BodyFacts::new(body, types)).collect();
        // A least fixpoint from nothing: every round can only add writes, and
        // paths are bounded, so it settles; should it not within the bound,
        // every function is taken to write anywhere.
        let mut rounds = 0;
        loop {
            rounds += 1;
            if rounds > MAX_ROUNDS {
                for writes in summaries.by_symbol.values_mut() {
                    writes.anywhere = true;
                }
                summaries.everything.anywhere = true;
                break;
            }
            let mut changed = false;
            // Every body with code, joined per symbol: bodies can share one
            // (a foreign declaration and its wrapper), and each adds to it.
            let mut next: HashMap<String, Writes> =
                summaries.by_symbol.keys().map(|symbol| (symbol.clone(), Writes::default())).collect();
            for (body, facts) in bodies.iter().zip(&facts) {
                if body.is_extern_declaration || body.is_abstract {
                    continue;
                }
                let Some(writes) = next.get_mut(&body.symbol) else { continue };
                for block in &body.blocks {
                    let mut out = Vec::new();
                    block_writes(body, types, &summaries, &facts.origins, block, &mut out);
                    for loc in out {
                        writes.add(loc);
                    }
                }
            }
            if next != summaries.by_symbol {
                summaries.by_symbol = next;
                changed = true;
            }
            let mut everything = Writes::default();
            for writes in summaries.by_symbol.values() {
                everything.join(writes);
            }
            everything.params.clear();
            if everything != summaries.everything {
                summaries.everything = everything;
                changed = true;
            }
            if !changed {
                break;
            }
        }
        summaries
    }

    fn of(&self, symbol: &str) -> Option<&Writes> {
        self.by_symbol.get(symbol)
    }

    /// The union of every body that can fill `slot` for a receiver of
    /// `owner`'s family; `None` when none is known.
    fn overrides(&self, types: &TypeTable, owner: ClassId, slot: usize) -> Option<Writes> {
        let family = class_family(types, owner);
        let mut writes = Writes::default();
        let mut found = false;
        for (class, method_slot, symbol) in &self.methods {
            if *method_slot == slot && class_family(types, *class) == family {
                writes.join(self.of(symbol)?);
                found = true;
            }
        }
        found.then_some(writes)
    }

    /// What dropping a `ty` can write, through every `drop` function it can
    /// run: its own, its bases', and those of everything it owns.
    fn drop_writes(&self, types: &TypeTable, ty: Ty) -> Writes {
        let mut writes = Writes::default();
        let mut seen = HashSet::new();
        self.drop_writes_into(types, ty, &mut writes, &mut seen);
        // A `drop` writes its own receiver, which is the value being dropped.
        writes.params.clear();
        writes
    }

    fn drop_writes_into(&self, types: &TypeTable, ty: Ty, writes: &mut Writes, seen: &mut HashSet<Ty>) {
        if !seen.insert(ty) || !types.needs_drop(ty) {
            return;
        }
        let own = |owner: String, writes: &mut Writes| match self.of(&ember_branding::drop_function(&owner)) {
            Some(drop) => writes.join(drop),
            None => writes.anywhere = true,
        };
        match types.kind(ty) {
            TyKind::Class(class) => {
                let mut current = Some(*class);
                while let Some(class) = current {
                    let def = types.class_def(class);
                    if def.has_drop {
                        own(def.name.to_string(), writes);
                    }
                    for field in def.fields.clone() {
                        self.drop_writes_into(types, field.ty, writes, seen);
                    }
                    current = def.base;
                }
            }
            TyKind::Struct(id) => {
                let def = types.struct_def(*id);
                if def.has_drop {
                    own(types.symbol_name(ty), writes);
                }
                for field in def.fields.clone() {
                    self.drop_writes_into(types, field.ty, writes, seen);
                }
                if let Some((_, args)) = def.origin.clone() {
                    for arg in args {
                        self.drop_writes_into(types, arg, writes, seen);
                    }
                }
            }
            TyKind::Enum(id) => {
                let def = types.enum_def(*id);
                if def.has_drop {
                    own(types.symbol_name(ty), writes);
                }
                for variant in def.variants.clone() {
                    for field in variant.fields {
                        self.drop_writes_into(types, field.ty, writes, seen);
                    }
                }
            }
            TyKind::Tuple(items) => {
                for item in items.clone() {
                    self.drop_writes_into(types, item, writes, seen);
                }
            }
            TyKind::Array { elem, .. } | TyKind::Vec { elem, .. } => self.drop_writes_into(types, *elem, writes, seen),
            // A `dyn` value or an interface handle can be any class: every
            // function in the program.
            _ => writes.join(&self.everything),
        }
    }
}

/// Each local assigned exactly once in the loop, with its block and value.
enum Definition {
    Value(Rvalue),
    Checked { op: BinOp, lhs: Operand, rhs: Operand },
}

fn definitions(body: &Body, blocks: &[usize]) -> HashMap<LocalId, (usize, Definition)> {
    let mut found: HashMap<LocalId, Option<(usize, Definition)>> = HashMap::new();
    fn record(
        found: &mut HashMap<LocalId, Option<(usize, Definition)>>,
        local: LocalId,
        block: usize,
        definition: Definition,
    ) {
        found.entry(local).and_modify(|slot| *slot = None).or_insert(Some((block, definition)));
    }
    for &block in blocks {
        for stmt in &body.blocks[block].stmts {
            match &stmt.kind {
                StmtKind::Assign { place, rvalue } if place.projection.is_empty() => {
                    record(&mut found, place.local, block, Definition::Value(rvalue.clone()));
                }
                StmtKind::CheckedBinaryOp { dest, op, lhs, rhs, .. } if dest.projection.is_empty() => {
                    let definition = Definition::Checked { op: *op, lhs: lhs.clone(), rhs: rhs.clone() };
                    record(&mut found, dest.local, block, definition);
                }
                _ => {}
            }
        }
        if let Terminator::Call { dest, .. } = &body.blocks[block].terminator {
            found.insert(dest.local, None);
        }
    }
    found.into_iter().filter_map(|(local, slot)| slot.map(|slot| (local, slot))).collect()
}

/// `local` as `counter + offset`. Each step is defined once in the loop, in a
/// block dominating `use_block`: a copy of a local of the same type, an add or
/// subtract of a constant in the counter's type, or the one conversion of the
/// counter's type to `usize` (the entry test makes that value non-negative).
#[allow(clippy::too_many_arguments)]
fn affine(
    body: &Body,
    types: &TypeTable,
    definitions: &HashMap<LocalId, (usize, Definition)>,
    dominators: &HashMap<usize, HashSet<usize>>,
    counter: LocalId,
    local: LocalId,
    use_block: usize,
    depth: usize,
) -> Option<i128> {
    if local == counter {
        return Some(0);
    }
    if depth > 16 {
        return None;
    }
    let (block, definition) = definitions.get(&local)?;
    if !dominators.get(&use_block)?.contains(block) {
        return None;
    }
    let counter_ty = body.local(counter).ty;
    let ty = body.local(local).ty;
    let from = |operand: &Operand, want: Ty| match operand {
        Operand::Copy(place) | Operand::Move(place)
            if place.projection.is_empty() && body.local(place.local).ty == want =>
        {
            affine(body, types, definitions, dominators, counter, place.local, *block, depth + 1)
        }
        _ => None,
    };
    let constant = |operand: &Operand| match operand {
        Operand::Const(Const::Int { value, ty }) if *ty == counter_ty => Some(constant_value(types, *value, counter_ty)),
        _ => None,
    };
    match definition {
        Definition::Value(Rvalue::Use(operand)) => from(operand, ty),
        Definition::Value(Rvalue::Cast { kind: CastKind::Numeric, operand, .. })
            if matches!(types.kind(ty), TyKind::Uint(ember_types::UintTy::Usize)) =>
        {
            from(operand, counter_ty)
        }
        Definition::Value(Rvalue::BinaryOp { op, lhs, rhs }) | Definition::Checked { op, lhs, rhs }
            if ty == counter_ty =>
        {
            match op {
                BinOp::Add => from(lhs, ty)
                    .zip(constant(rhs))
                    .or_else(|| from(rhs, ty).zip(constant(lhs)))
                    .map(|(a, c)| a + c),
                BinOp::Sub => from(lhs, ty).zip(constant(rhs)).map(|(a, c)| a - c),
                _ => None,
            }
        }
        _ => None,
    }
}

/// A constant's two's-complement bits as a number of type `ty`.
fn constant_value(types: &TypeTable, value: u128, ty: Ty) -> i128 {
    let width = ember_types::bit_width(types, ty).unwrap_or(64).min(64) as u32;
    let low = if width == 64 { value as u64 } else { (value as u64) & ((1u64 << width) - 1) };
    if ember_types::is_signed(types, ty) == Some(true) {
        ((low << (64 - width)) as i64 >> (64 - width)) as i128
    } else {
        low as i128
    }
}

fn predecessors(body: &Body) -> Vec<Vec<usize>> {
    let mut predecessors = vec![Vec::new(); body.blocks.len()];
    for (index, block) in body.blocks.iter().enumerate() {
        for successor in successors(&block.terminator) {
            predecessors[successor.0 as usize].push(index);
        }
    }
    predecessors
}

fn successors(terminator: &Terminator) -> Vec<BasicBlockId> {
    match terminator {
        Terminator::Goto(target) => vec![*target],
        Terminator::SwitchInt { targets, otherwise, .. } => {
            targets.iter().map(|(_, target)| *target).chain(std::iter::once(*otherwise)).collect()
        }
        Terminator::Call { next, .. } | Terminator::Assert { next, .. } => vec![*next],
        Terminator::Return | Terminator::Unreachable => Vec::new(),
    }
}

pub(crate) fn retarget(terminator: &mut Terminator, map: impl Fn(BasicBlockId) -> BasicBlockId) {
    match terminator {
        Terminator::Goto(target) => *target = map(*target),
        Terminator::SwitchInt { targets, otherwise, .. } => {
            for (_, target) in targets.iter_mut() {
                *target = map(*target);
            }
            *otherwise = map(*otherwise);
        }
        Terminator::Call { next, .. } | Terminator::Assert { next, .. } => *next = map(*next),
        Terminator::Return | Terminator::Unreachable => {}
    }
}

fn reachable_from(body: &Body, start: usize, stop: usize) -> HashSet<usize> {
    let mut seen = HashSet::new();
    let mut work = vec![start];
    while let Some(block) = work.pop() {
        if block == stop || !seen.insert(block) {
            continue;
        }
        work.extend(successors(&body.blocks[block].terminator).into_iter().map(|b| b.0 as usize));
    }
    seen
}

/// Dominators within the loop, the header as the entry.
fn dominators(body: &Body, header: usize, blocks: &[usize]) -> HashMap<usize, HashSet<usize>> {
    let all: HashSet<usize> = blocks.iter().copied().collect();
    let predecessors = predecessors(body);
    let mut dominators: HashMap<usize, HashSet<usize>> =
        blocks.iter().map(|&b| (b, if b == header { HashSet::from([header]) } else { all.clone() })).collect();
    let mut changed = true;
    while changed {
        changed = false;
        for &block in blocks {
            if block == header {
                continue;
            }
            let mut next: Option<HashSet<usize>> = None;
            for p in predecessors[block].iter().filter(|p| all.contains(p)) {
                let set = &dominators[p];
                next = Some(match next {
                    None => set.clone(),
                    Some(current) => current.intersection(set).copied().collect(),
                });
            }
            let mut next = next.unwrap_or_default();
            next.insert(block);
            if next != dominators[&block] {
                dominators.insert(block, next);
                changed = true;
            }
        }
    }
    dominators
}

/// Returns the header of the unchecked copy.
fn apply(body: &mut Body, types: &TypeTable, plan: Plan) -> usize {
    let span = body.blocks[plan.header].terminator_span;
    let mut copied: Vec<usize> = plan.region.clone();
    copied.push(plan.header);
    let base = body.blocks.len();
    let map: HashMap<usize, usize> = copied.iter().enumerate().map(|(i, &b)| (b, base + i)).collect();
    let remap = |target: BasicBlockId| map.get(&(target.0 as usize)).map_or(target, |&n| BasicBlockId(n as u32));

    // The unchecked copy: the loop's blocks with the planned checks removed.
    for &block in &copied {
        let mut clone: BasicBlock = body.blocks[block].clone();
        retarget(&mut clone.terminator, remap);
        if plan.checks.iter().any(|check| check.block == block) {
            let Terminator::Assert { next, cond, .. } = clone.terminator else {
                unreachable!("a planned check is an assert")
            };
            // `[EFF-10]` — the check now runs once, before the loop.
            body.removed_checks.push(RemovedCheck {
                span: clone.terminator_span,
                kind: CheckKind::Bounds,
                proof: CheckProof::LoopEntryTest { loop_span: span },
            });
            // The comparison that fed the check goes with it.
            if let Operand::Copy(cond) = cond {
                if let Some(stmt) = clone.stmts.iter_mut().rev().find(|stmt| {
                    matches!(&stmt.kind, StmtKind::Assign { place, .. } if cond.projection.is_empty() && *place == cond)
                }) {
                    stmt.kind = StmtKind::Nop;
                }
            }
            clone.terminator = Terminator::Goto(next);
        }
        body.blocks.push(clone);
    }
    let fast = remap(BasicBlockId(plan.header as u32));
    let checked = BasicBlockId(plan.header as u32);

    // The entry test: one block per condition, each going to the checked loop
    // when its condition fails, the last to the unchecked loop.
    let mut tests = entry_tests(body, types, &plan, span, checked);

    // In the unchecked copy each view's header is read once, into a local of
    // its own that nothing else can reach, and its elements are reached
    // through that: the same memory, but a base the C compiler can keep in a
    // register, as a loop over a C pointer has.
    let mut hoisted: Vec<(Place, LocalId)> = Vec::new();
    for check in &plan.checks {
        let Operand::Copy(len) = &check.len else { continue };
        let view = Place { local: len.local, projection: len.projection[..len.projection.len() - 1].to_vec() };
        if hoisted.iter().any(|(seen, _)| *seen == view) {
            continue;
        }
        let ty = place_type(body, types, &view);
        body.locals.push(LocalDecl { ty, kind: LocalKind::Temp, name: None, span });
        let local = LocalId(body.locals.len() as u32 - 1);
        let last = tests.last_mut().expect("the entry test has at least the run test");
        last.0.push(Stmt::new(
            StmtKind::Assign { place: Place::local(local), rvalue: Rvalue::Use(Operand::Copy(view.clone())) },
            span,
        ));
        hoisted.push((view, local));
    }
    let through_hoisted = |place: &mut Place| {
        for (view, local) in &hoisted {
            let n = view.projection.len();
            if place.local == view.local
                && place.projection.len() > n
                && place.projection[..n] == view.projection[..]
                && matches!(place.projection[n], Projection::Index(_) | Projection::ConstIndex(_))
            {
                place.local = *local;
                place.projection.drain(..n);
                return;
            }
        }
    };
    for index in base..base + copied.len() {
        rewrite_places(&mut body.blocks[index], &through_hoisted);
    }

    let entry = BasicBlockId(body.blocks.len() as u32);
    let count = tests.len();
    for (index, (stmts, condition, fail)) in tests.into_iter().enumerate() {
        let next = if index + 1 == count { fast } else { BasicBlockId(entry.0 + index as u32 + 1) };
        body.blocks.push(BasicBlock {
            stmts,
            terminator: Terminator::SwitchInt {
                discr: Operand::Copy(Place::local(condition)),
                targets: vec![(0, fail)],
                otherwise: next,
            },
            terminator_span: span,
        });
    }

    // Enter through the test.
    let inside: HashSet<usize> = copied.iter().copied().collect();
    for index in (0..base).filter(|index| !inside.contains(index)) {
        retarget(&mut body.blocks[index].terminator, |target| if target == checked { entry } else { target });
    }
    fast.0 as usize
}

/// The entry test's conditions in order, each with where it goes when false.
/// Every operation is proved not to overflow by the conditions before it.
fn entry_tests(
    body: &mut Body,
    types: &TypeTable,
    plan: &Plan,
    span: ember_span::Span,
    checked: BasicBlockId,
) -> Vec<(Vec<Stmt>, LocalId, BasicBlockId)> {
    let counter_ty = body.local(plan.counter).ty;
    let bool_ty = match &body.blocks[plan.header].terminator {
        Terminator::SwitchInt { discr: Operand::Copy(discr), .. } => body.local(discr.local).ty,
        _ => unreachable!("a planned loop header branches on its test"),
    };
    let signed = ember_types::is_signed(types, counter_ty) == Some(true);
    let width = ember_types::bit_width(types, counter_ty).unwrap_or(64);
    let max: i128 = if signed { (1i128 << (width - 1)) - 1 } else { (1i128 << width) - 1 };
    let int = |value: i128| Operand::Const(Const::Int { value: value as u128, ty: counter_ty });
    let counter = || Operand::Copy(Place::local(plan.counter));
    let limit = || Operand::Copy(Place::local(plan.limit));
    let temp = |body: &mut Body, ty: Ty| {
        body.locals.push(LocalDecl { ty, kind: LocalKind::Temp, name: None, span });
        LocalId(body.locals.len() as u32 - 1)
    };
    let assign = |local: LocalId, rvalue: Rvalue| Stmt::new(StmtKind::Assign { place: Place::local(local), rvalue }, span);
    let compare = |op: BinOp, lhs: Operand, rhs: Operand| Rvalue::BinaryOp { op, lhs, rhs };

    let mut tests = Vec::new();
    // Whether the loop runs at all; when it does not, the checked copy does
    // nothing, and the unchecked copy is entered only past the whole test.
    let run = temp(body, bool_ty);
    let op = if plan.inclusive { BinOp::Le } else { BinOp::Lt };
    tests.push((vec![assign(run, compare(op, counter(), limit()))], run, checked));

    let mut seen: Vec<(IndexBound, String)> = Vec::new();
    for check in &plan.checks {
        let key = (check.bound, format!("{:?}", check.len));
        if seen.contains(&key) {
            continue;
        }
        seen.push(key);
        let offset = match check.bound {
            IndexBound::Counter(offset) => offset,
            IndexBound::Constant(hi) => {
                // `[RNG-4]` — every index is at most `hi`: below the length
                // when `hi` is.
                let fits = temp(body, bool_ty);
                let hi = Operand::Const(Const::Int { value: hi as u128, ty: check.index_ty });
                tests.push((vec![assign(fits, compare(BinOp::Lt, hi, check.len.clone()))], fits, checked));
                continue;
            }
            IndexBound::Local(local, c) => {
                let local_ty = body.local(local).ty;
                let signed = ember_types::is_signed(types, local_ty) == Some(true);
                let width = ember_types::bit_width(types, local_ty).unwrap_or(64);
                let max: i128 = if signed { (1i128 << (width - 1)) - 1 } else { (1i128 << width) - 1 };
                let value = |v: i128| Operand::Const(Const::Int { value: v as u128, ty: local_ty });
                let bound = || Operand::Copy(Place::local(local));
                // `local + c` is not negative, and computing it cannot overflow.
                if signed || c < 0 {
                    let low = temp(body, bool_ty);
                    tests.push((vec![assign(low, compare(BinOp::Ge, bound(), value(-c)))], low, checked));
                }
                if c > 0 {
                    let room = temp(body, bool_ty);
                    tests.push((vec![assign(room, compare(BinOp::Le, bound(), value(max - c)))], room, checked));
                }
                let mut stmts = Vec::new();
                let end = if c == 0 {
                    bound()
                } else {
                    let end = temp(body, local_ty);
                    let (op, amount) = if c > 0 { (BinOp::Add, c) } else { (BinOp::Sub, -c) };
                    stmts.push(assign(end, compare(op, bound(), value(amount))));
                    Operand::Copy(Place::local(end))
                };
                let end = if local_ty == check.index_ty {
                    end
                } else {
                    let wide = temp(body, check.index_ty);
                    stmts.push(assign(wide, Rvalue::Cast { kind: CastKind::Numeric, operand: end, to: check.index_ty }));
                    Operand::Copy(Place::local(wide))
                };
                let fits = temp(body, bool_ty);
                stmts.push(assign(fits, compare(BinOp::Lt, end, check.len.clone())));
                tests.push((stmts, fits, checked));
                continue;
            }
        };
        // The first index, `a + c`, is not negative. The loop runs, so
        // `b > a` (`b >= a` for `a..=b`) and the last index is not either.
        if offset < 0 || signed {
            let low = temp(body, bool_ty);
            tests.push((vec![assign(low, compare(BinOp::Ge, counter(), int(-offset)))], low, checked));
        }
        // `b + c` is computed only once it cannot overflow.
        if offset > 0 {
            let room = temp(body, bool_ty);
            tests.push((vec![assign(room, compare(BinOp::Le, limit(), int(max - offset)))], room, checked));
        }
        // The last index, `b - 1 + c` (`b + c` for `a..=b`), is below the length.
        let len_ty = check.index_ty;
        let mut stmts = Vec::new();
        let end = if offset == 0 {
            limit()
        } else {
            let end = temp(body, counter_ty);
            let (op, amount) = if offset > 0 { (BinOp::Add, offset) } else { (BinOp::Sub, -offset) };
            stmts.push(assign(end, compare(op, limit(), int(amount))));
            Operand::Copy(Place::local(end))
        };
        let end = if counter_ty == len_ty {
            end
        } else {
            let wide = temp(body, len_ty);
            stmts.push(assign(wide, Rvalue::Cast { kind: CastKind::Numeric, operand: end, to: len_ty }));
            Operand::Copy(Place::local(wide))
        };
        let fits = temp(body, bool_ty);
        let op = if plan.inclusive { BinOp::Lt } else { BinOp::Le };
        stmts.push(assign(fits, compare(op, end, check.len.clone())));
        tests.push((stmts, fits, checked));
    }
    tests
}

/// Apply `f` to every place a block names.
fn rewrite_places(block: &mut BasicBlock, f: &impl Fn(&mut Place)) {
    let operand = |operand: &mut Operand| {
        if let Operand::Copy(place) | Operand::Move(place) = operand {
            f(place);
        }
    };
    for stmt in &mut block.stmts {
        match &mut stmt.kind {
            StmtKind::Assign { place, rvalue } => {
                f(place);
                match rvalue {
                    Rvalue::Use(value) | Rvalue::UnaryOp { operand: value, .. } | Rvalue::Cast { operand: value, .. } => {
                        operand(value)
                    }
                    Rvalue::BinaryOp { lhs, rhs, .. } => {
                        operand(lhs);
                        operand(rhs);
                    }
                    Rvalue::Aggregate { operands, .. } => operands.iter_mut().for_each(operand),
                    Rvalue::Repeat { value, .. } => operand(value),
                    Rvalue::Discriminant(place) | Rvalue::Ref { place, .. } => f(place),
                }
            }
            StmtKind::CheckedBinaryOp { dest, overflow, lhs, rhs, .. } => {
                f(dest);
                f(overflow);
                operand(lhs);
                operand(rhs);
            }
            StmtKind::Drop { place, .. }
            | StmtKind::BeginAccess { place, .. }
            | StmtKind::BeginAccessTransfer { place, .. }
            | StmtKind::EndAccess { place, .. }
            | StmtKind::EndAccessTransfer { place, .. } => f(place),
            StmtKind::StorageLive(_) | StmtKind::StorageDead(_) | StmtKind::Nop => {}
        }
    }
    match &mut block.terminator {
        Terminator::SwitchInt { discr, .. } => operand(discr),
        Terminator::Call { func, args, dest, .. } => {
            if let FuncRef::Indirect { operand: callee, .. } = func {
                operand(callee);
            }
            args.iter_mut().for_each(operand);
            f(dest);
        }
        Terminator::Assert { cond, msg, .. } => {
            operand(cond);
            match msg {
                AssertKind::Bounds { len, index } => {
                    operand(len);
                    operand(index);
                }
                AssertKind::RefCellBorrow { file, line } => {
                    operand(file);
                    operand(line);
                }
                AssertKind::Panic { message } => operand(message),
                _ => {}
            }
        }
        Terminator::Goto(_) | Terminator::Return | Terminator::Unreachable => {}
    }
}

// ---------------------------------------------------------------------------
// `[SIMD-7]` — grouped overflow checks.
//
// A counted loop in vectorisable form (`[SIMD-5]`, below) runs in groups of
// `group_size` iterations. Each `+` and `-` checked for overflow computes its
// wrapped result and an overflow bit with plain arithmetic, and at the end of
// the group a set bit panics with the location of the first overflowing
// iteration's first overflowing operation: exactly the panic the loop would
// have raised. How the group finds that location (`group`,
// `detect_then_commit`):
// - report, when the loop has one check: its flag panics with that check's
//   location at the group's end;
// - re-run, with several: one flag; a set flag restores the loop's locals
//   and re-runs the group checked. A view the loop both reads and writes has
//   the group's elements saved at its start and put back first;
// - detect, then commit, when such a view is not accessed on every iteration
//   (so the group's elements are not all known to exist): a first run that
//   stores nothing sets the flag, then a set flag re-runs the group checked
//   and a clear one runs it again for real with no check;
// - bits, where detect-then-commit cannot place its loads: a mask per
//   operation, a bit per iteration.
// Iterations left over after the last full group run in the loop as it was,
// checked one operation at a time.
//
// Before anything in a group that could panic another way (a dynamic
// exclusivity check) or run long (an inner loop), the masks are tested first,
// so an earlier overflow is never overtaken.
//
// An integer running total (`total = total ± x`) is not grouped (ODR-086). A
// total of at most 32-bit elements widened into a 64-bit total gets a copy with
// no overflow check at all, taken when the trip count and the total's value at
// entry prove no partial sum can overflow in any grouping. Otherwise a signed
// total of at least 16 bits is proved safe at run time, block by block: in
// blocks of 64 iterations, the total checked small enough at the block's start
// and every value it adds checked small enough during the block
// (`block_totals`).
//
// Vectorisable form, computed here and never delegated to the C compiler:
// - a counted loop whose trip count is known at entry, with one exit;
// - no bounds check left (`[OPT-2]` removed them all);
// - every memory access is to a local, or to a view held in a local at the
//   counter plus a constant, each written view at one offset only;
// - every written view is disjoint from every other view the loop accesses
//   (distinct owners, distinct fields, or borrows the borrow rules keep apart);
// - no call: a call is read through only into a function `[CG-C-3]` places in
//   the inline header, and that header is not built (`docs/NOT-IMPLEMENTED.md`);
// - no drop that runs anything, no explicit panic, and no check but an
//   integer overflow check;
// - no floating-point running total (`@fastmath` and `@parallel(reduce=…)`,
//   which would declare one, are not built).

/// Iterations per group, measured on list loops (ADR-070 item 7). With one
/// check, 32 was the fastest on MSVC and on clang (at 16 MSVC spent 5% more
/// than clang setting up each group). With several, 16 kept the slower of the
/// two compilers fastest. A bits group is at most 64: one bit per iteration.
fn group_size(mode: Mode) -> u64 {
    if mode == Mode::Report { 32 } else { 16 }
}

/// Largest trip count the running-total proof accepts.
const TOTAL_TRIPS: i128 = 1 << 31;

/// The canonical shape `lower_for_range` emits.
pub(crate) struct CountedLoop {
    pub(crate) header: usize,
    entry: usize,
    pub(crate) step: usize,
    pub(crate) region: Vec<usize>,
    pub(crate) counter: LocalId,
    pub(crate) limit: LocalId,
    pub(crate) inclusive: bool,
}

pub(crate) fn counted_loop(body: &Body, types: &TypeTable, header: usize) -> Option<CountedLoop> {
    let head = body.blocks.get(header)?;
    let Terminator::SwitchInt { discr: Operand::Copy(discr), targets, otherwise } = &head.terminator else {
        return None;
    };
    if !discr.projection.is_empty() || targets.len() != 1 || targets[0].0 != 0 {
        return None;
    }
    let (op, counter, limit) = head.stmts.iter().rev().find_map(|stmt| match &stmt.kind {
        StmtKind::Assign {
            place,
            rvalue: Rvalue::BinaryOp { op: op @ (BinOp::Lt | BinOp::Le), lhs: Operand::Copy(lhs), rhs: Operand::Copy(rhs) },
        } if *place == Place::local(discr.local) && lhs.projection.is_empty() && rhs.projection.is_empty() => {
            Some((*op, lhs.local, rhs.local))
        }
        _ => None,
    })?;
    let counter_ty = body.local(counter).ty;
    let width = ember_types::bit_width(types, counter_ty)?;
    if !(32..=64).contains(&width) || body.local(limit).ty != counter_ty {
        return None;
    }
    let predecessors = predecessors(body);
    let entry = otherwise.0 as usize;
    let reach = reachable_from(body, entry, header);
    let back: Vec<usize> = predecessors[header].iter().copied().filter(|p| reach.contains(p)).collect();
    let [step] = back.as_slice() else { return None };
    let mut region: HashSet<usize> = HashSet::new();
    let mut work = vec![*step];
    while let Some(block) = work.pop() {
        if block == header || !region.insert(block) {
            continue;
        }
        work.extend(predecessors[block].iter().copied());
    }
    if !region.contains(&entry) {
        return None;
    }
    if region.iter().any(|&b| predecessors[b].iter().any(|p| *p != header && !region.contains(p))) {
        return None;
    }
    let step_block = &body.blocks[*step];
    let increments = step_block
        .stmts
        .iter()
        .filter(|stmt| matches!(&stmt.kind, StmtKind::Assign { place, .. } if place.local == counter))
        .count();
    let is_increment = step_block.stmts.iter().any(|stmt| {
        matches!(&stmt.kind,
            StmtKind::Assign { place, rvalue: Rvalue::BinaryOp { op: BinOp::Add, lhs: Operand::Copy(lhs), rhs: Operand::Const(Const::Int { value: 1, .. }) } }
                if *place == Place::local(counter) && *lhs == Place::local(counter))
    });
    if increments != 1 || !is_increment || !matches!(step_block.terminator, Terminator::Goto(target) if target.0 as usize == header) {
        return None;
    }
    let mut region: Vec<usize> = region.into_iter().collect();
    region.sort_unstable();
    Some(CountedLoop { header, entry, step: *step, region, counter, limit, inclusive: op == BinOp::Le })
}

/// Group the overflow checks of every loop of `body` in vectorisable form.
fn group_overflow_checks(body: &mut Body, types: &TypeTable, common: &CommonTypes) -> usize {
    let mut count = 0;
    let snapshot = body.blocks.len();
    for header in 0..snapshot {
        let Some(shape) = counted_loop(body, types, header) else { continue };
        let facts = BodyFacts::new(body, types);
        let Some((checks, written)) = vectorisable(body, types, &facts, &shape) else { continue };
        if checks.is_empty() {
            continue;
        }
        // A running total (`acc = acc ± x`) is vectorised under checked
        // arithmetic only when no partial sum can overflow in any grouping:
        // then it gets a copy with no check. Otherwise its checks stay one
        // per operation (the reading of `[SIMD-7]` awaiting the owner's
        // ruling; see ADR-070).
        if checks.iter().any(|check| running_total(&body.blocks[check.block].stmts[check.stmt])) {
            if let Some(entry) = unchecked_totals(body, types, common, &shape, &checks, BasicBlockId(shape.header as u32)) {
                enter_through(body, &shape, entry);
                count += 1;
                continue;
            }
            // Otherwise the totals are proved safe block by block at run time
            // (ODR-086), the other checks grouped beside them; a total that
            // cannot be keeps a check on every operation.
            let (Some(totals), Some(saves)) =
                (block_totals(body, types, &shape, &checks), undo_saves(body, types, &shape, &written))
            else {
                continue;
            };
            group(body, types, common, &shape, &checks, Mode::Rerun, &saves, &totals);
            count += 1;
            continue;
        }
        if checks.len() == 1 {
            group(body, types, common, &shape, &checks, Mode::Report, &[], &[]);
        } else if let Some(saves) = undo_saves(body, types, &shape, &written) {
            group(body, types, common, &shape, &checks, Mode::Rerun, &saves, &[]);
        } else {
            let origins: Vec<Place> = written.iter().map(|view| view.origin.clone()).collect();
            if !detect_then_commit(body, types, common, &shape, &checks, &origins) {
                group(body, types, common, &shape, &checks, Mode::Bits, &[], &[]);
            }
        }
        count += 1;
    }
    count
}

/// One overflow check of the loop: the block whose terminator is its assert,
/// the checked operation's statement, and the assert's message and span.
#[derive(Clone)]
struct OverflowCheck {
    block: usize,
    stmt: usize,
    msg: AssertKind,
    span: ember_span::Span,
}

/// A view a vectorisable loop writes: the place `vf_place` names it by, the
/// one offset from the counter it is accessed at, and whether the loop also
/// reads it.
#[derive(Clone)]
struct WrittenView {
    origin: Place,
    offset: i128,
    read: bool,
}

/// The loop's overflow checks in execution order when it is in vectorisable
/// form, and the views it writes; `None` when it is not in that form.
fn vectorisable(
    body: &Body,
    types: &TypeTable,
    facts: &BodyFacts,
    shape: &CountedLoop,
) -> Option<(Vec<OverflowCheck>, Vec<WrittenView>)> {
    // `[SIMD-5]` (ODR-088): a division no vector instruction set does keeps
    // the loop one iteration at a time.
    if scalar_only(body, types, shape) {
        return None;
    }
    let mut all = shape.region.clone();
    all.push(shape.header);
    let inside: HashSet<usize> = all.iter().copied().collect();
    let definitions = definitions(body, &all);
    let dominators = dominators(body, shape.header, &all);
    let mut checks = Vec::new();
    // Views accessed: the place each came from, its offsets, whether written,
    // whether read.
    let mut views: Vec<(Place, HashSet<i128>, bool, bool)> = Vec::new();

    for &block in &shape.region {
        let data = &body.blocks[block];
        // One exit: nothing in the body leaves the loop.
        if successors(&data.terminator).iter().any(|s| !inside.contains(&(s.0 as usize))) {
            return None;
        }
        match &data.terminator {
            Terminator::Goto(_) | Terminator::SwitchInt { .. } => {}
            Terminator::Assert { cond: Operand::Copy(cond), expected: false, msg: msg @ AssertKind::Overflow(_), span, .. }
                if cond.projection.is_empty() =>
            {
                let stmt = data.stmts.iter().rposition(|stmt| {
                    matches!(&stmt.kind, StmtKind::CheckedBinaryOp { overflow, .. } if *overflow == Place::local(cond.local))
                })?;
                checks.push(OverflowCheck { block, stmt, msg: msg.clone(), span: *span });
            }
            _ => return None,
        }
        for (index, stmt) in data.stmts.iter().enumerate() {
            match &stmt.kind {
                StmtKind::Assign { place, rvalue } => {
                    if matches!(rvalue, Rvalue::Ref { .. }) {
                        return None;
                    }
                    // A floating-point running total needs `@fastmath` or
                    // `@parallel(reduce=…)` to be reordered.
                    if types.is_float(place_type(body, types, place))
                        && place.projection.is_empty()
                        && rvalue_reads_local(rvalue, place.local)
                    {
                        return None;
                    }
                }
                StmtKind::CheckedBinaryOp { .. } | StmtKind::StorageLive(_) | StmtKind::StorageDead(_) | StmtKind::Nop => {}
                StmtKind::Drop { place, .. } => {
                    if types.needs_drop(place_type(body, types, place)) {
                        return None;
                    }
                }
                // Only a check: the begin directly followed by its end.
                StmtKind::BeginAccess { place, mutable } => {
                    let paired = matches!(
                        data.stmts.get(index + 1).map(|next| &next.kind),
                        Some(StmtKind::EndAccess { place: end, mutable: end_mutable }) if end == place && end_mutable == mutable
                    );
                    if !paired {
                        return None;
                    }
                }
                StmtKind::EndAccess { place, mutable } => {
                    let paired = index > 0
                        && matches!(
                            &data.stmts[index - 1].kind,
                            StmtKind::BeginAccess { place: begin, mutable: begin_mutable } if begin == place && begin_mutable == mutable
                        );
                    if !paired {
                        return None;
                    }
                }
                StmtKind::BeginAccessTransfer { .. } | StmtKind::EndAccessTransfer { .. } => return None,
            }
        }
        // Every memory access: a local, or a view held in a local at the
        // counter plus a constant.
        let mut ok = true;
        visit_places(data, &mut |place: &Place, written: bool, access_statement: bool| {
            if !ok || access_statement {
                return;
            }
            match vf_place(body, types, &definitions, &dominators, shape.counter, block, place) {
                Err(()) => ok = false,
                Ok(None) => {}
                Ok(Some((origin, offset))) => match views.iter_mut().find(|(seen, _, _, _)| *seen == origin) {
                    Some((_, offsets, was_written, was_read)) => {
                        offsets.insert(offset);
                        *was_written |= written;
                        *was_read |= !written;
                    }
                    None => views.push((origin, HashSet::from([offset]), written, !written)),
                },
            }
        });
        if !ok {
            return None;
        }
    }
    // Each written view at one offset, and disjoint from every other view.
    for (index, (origin, offsets, written, _)) in views.iter().enumerate() {
        if !*written {
            continue;
        }
        if offsets.len() != 1 {
            return None;
        }
        let here = locate(body, types, &facts.origins, origin, 0)?;
        for (other_index, (other, _, _, _)) in views.iter().enumerate() {
            if other_index == index {
                continue;
            }
            let there = locate(body, types, &facts.origins, other, 0)?;
            if !disjoint(&here, &there) {
                return None;
            }
        }
    }
    // Execution order: the order the body's blocks are first reached from
    // its entry, then statement order.
    let order = reach_order(body, shape.entry, shape.header);
    checks.sort_by_key(|check| (order.iter().position(|b| *b == check.block).unwrap_or(usize::MAX), check.stmt));
    let written = views
        .into_iter()
        .filter(|(_, _, written, _)| *written)
        .map(|(origin, offsets, _, read)| {
            let offset = *offsets.iter().next().expect("a written view has its one offset");
            WrittenView { origin, offset, read }
        })
        .collect();
    Some((checks, written))
}

fn rvalue_reads_local(rvalue: &Rvalue, local: LocalId) -> bool {
    let reads = |operand: &Operand| matches!(operand, Operand::Copy(place) | Operand::Move(place) if place.local == local);
    match rvalue {
        Rvalue::Use(operand) | Rvalue::UnaryOp { operand, .. } | Rvalue::Cast { operand, .. } => reads(operand),
        Rvalue::BinaryOp { lhs, rhs, .. } => reads(lhs) || reads(rhs),
        Rvalue::Aggregate { operands, .. } => operands.iter().any(reads),
        Rvalue::Repeat { value, .. } => reads(value),
        Rvalue::Discriminant(place) | Rvalue::Ref { place, .. } => place.local == local,
    }
}

/// Two views' memory cannot overlap: distinct owners, distinct fields, or
/// borrows the borrow rules keep apart.
fn disjoint(a: &Loc, b: &Loc) -> bool {
    let related = |x: &[Step], y: &[Step]| {
        let n = x.len().min(y.len());
        x[..n] == y[..n]
    };
    match (&a.root, &b.root) {
        (Root::Anywhere, _) | (_, Root::Anywhere) => false,
        (Root::Local(x), Root::Local(y)) => x != y || !related(&a.path, &b.path),
        (Root::Param(x), Root::Param(y)) => x != y || !related(&a.path, &b.path),
        (Root::Heap(x), Root::Heap(y)) => x != y || !related(&a.path, &b.path),
        _ => true,
    }
}

/// How `[SIMD-5]` sees one place: `Ok(None)` for a local, `Ok(Some((view,
/// offset)))` for a view element at the counter plus `offset` (the view being
/// the place the local holding it was copied from, if it was), `Err` for any
/// other memory.
fn vf_place(
    body: &Body,
    types: &TypeTable,
    definitions: &HashMap<LocalId, (usize, Definition)>,
    dominators: &HashMap<usize, HashSet<usize>>,
    counter: LocalId,
    block: usize,
    place: &Place,
) -> Result<Option<(Place, i128)>, ()> {
    let mut index_at = None;
    for (position, projection) in place.projection.iter().enumerate() {
        let prefix = Place { local: place.local, projection: place.projection[..position].to_vec() };
        let prefix_ty = place_type(body, types, &prefix);
        match projection {
            Projection::Deref | Projection::Column(_) => return Err(()),
            Projection::Field(_) if matches!(types.kind(prefix_ty), TyKind::Class(_)) => return Err(()),
            Projection::Field(_) | Projection::Downcast(_) => {}
            Projection::ConstIndex(_) => {
                if index_at.is_some() || !matches!(types.kind(prefix_ty), TyKind::Array { .. }) {
                    return Err(());
                }
            }
            Projection::Index(local) => {
                if index_at.is_some() {
                    return Err(());
                }
                index_at = Some((position, *local));
            }
        }
    }
    let Some((position, index)) = index_at else { return Ok(None) };
    let view = Place { local: place.local, projection: place.projection[..position].to_vec() };
    if !matches!(types.kind(place_type(body, types, &view)), TyKind::Vec { .. } | TyKind::Span { .. } | TyKind::Array { .. }) {
        return Err(());
    }
    let offset = affine(body, types, definitions, dominators, counter, index, block, 0).ok_or(())?;
    // A local holding a copy of a view header (the unchecked copy's own)
    // stands for the view it was copied from.
    let origin = match (view.projection.is_empty(), copied_from(body, view.local)) {
        (true, Some(source)) => source,
        _ => view,
    };
    Ok(Some((origin, offset)))
}

/// The place a local was copied from, when its every assignment copies that
/// one place.
fn copied_from(body: &Body, local: LocalId) -> Option<Place> {
    let mut source: Option<Place> = None;
    for block in &body.blocks {
        for stmt in &block.stmts {
            if let StmtKind::Assign { place, rvalue } = &stmt.kind {
                if *place == Place::local(local) {
                    let Rvalue::Use(Operand::Copy(from)) = rvalue else { return None };
                    match &source {
                        Some(seen) if seen != from => return None,
                        _ => source = Some(from.clone()),
                    }
                }
            }
        }
    }
    source
}

/// Blocks reachable from `start` without passing `stop`, in first-reached order.
fn reach_order(body: &Body, start: usize, stop: usize) -> Vec<usize> {
    let mut order = Vec::new();
    let mut seen = HashSet::new();
    let mut work = std::collections::VecDeque::from([start]);
    while let Some(block) = work.pop_front() {
        if block == stop || !seen.insert(block) {
            continue;
        }
        order.push(block);
        work.extend(successors(&body.blocks[block].terminator).into_iter().map(|b| b.0 as usize));
    }
    order
}

/// Visit every place a block names: `(place, written, part of an access
/// check)`.
fn visit_places(block: &BasicBlock, f: &mut dyn FnMut(&Place, bool, bool)) {
    for stmt in &block.stmts {
        visit_stmt(stmt, f);
    }
    visit_terminator(&block.terminator, f);
}

fn visit_operand(operand: &Operand, f: &mut dyn FnMut(&Place, bool, bool)) {
    if let Operand::Copy(place) | Operand::Move(place) = operand {
        f(place, false, false);
    }
}

/// `visit_places` for one statement.
fn visit_stmt(stmt: &Stmt, f: &mut dyn FnMut(&Place, bool, bool)) {
    let operand = visit_operand;
    match &stmt.kind {
        StmtKind::Assign { place, rvalue } => {
            f(place, true, false);
            match rvalue {
                Rvalue::Use(value) | Rvalue::UnaryOp { operand: value, .. } | Rvalue::Cast { operand: value, .. } => {
                    operand(value, f)
                }
                Rvalue::BinaryOp { lhs, rhs, .. } => {
                    operand(lhs, f);
                    operand(rhs, f);
                }
                Rvalue::Aggregate { operands, .. } => {
                    for value in operands {
                        operand(value, f);
                    }
                }
                Rvalue::Repeat { value, .. } => operand(value, f),
                Rvalue::Discriminant(place) | Rvalue::Ref { place, .. } => f(place, false, false),
            }
        }
        StmtKind::CheckedBinaryOp { dest, overflow, lhs, rhs, .. } => {
            f(dest, true, false);
            f(overflow, true, false);
            operand(lhs, f);
            operand(rhs, f);
        }
        StmtKind::Drop { place, .. } => f(place, true, false),
        StmtKind::BeginAccess { place, .. }
        | StmtKind::BeginAccessTransfer { place, .. }
        | StmtKind::EndAccess { place, .. }
        | StmtKind::EndAccessTransfer { place, .. } => f(place, false, true),
        StmtKind::StorageLive(_) | StmtKind::StorageDead(_) | StmtKind::Nop => {}
    }
}

/// `visit_places` for a terminator.
fn visit_terminator(terminator: &Terminator, f: &mut dyn FnMut(&Place, bool, bool)) {
    let operand = visit_operand;
    match terminator {
        Terminator::SwitchInt { discr, .. } => operand(discr, f),
        Terminator::Call { args, dest, .. } => {
            for arg in args {
                operand(arg, f);
            }
            f(dest, true, false);
        }
        Terminator::Assert { cond, msg, .. } => {
            operand(cond, f);
            if let AssertKind::Bounds { len, index } = msg {
                operand(len, f);
                operand(index, f);
            }
        }
        Terminator::Goto(_) | Terminator::Return | Terminator::Unreachable => {}
    }
}

/// Small builders for the statements this pass adds.
struct Build {
    span: ember_span::Span,
}

impl Build {
    fn temp(&self, body: &mut Body, ty: Ty) -> LocalId {
        body.locals.push(LocalDecl { ty, kind: LocalKind::Temp, name: None, span: self.span });
        LocalId(body.locals.len() as u32 - 1)
    }

    fn assign(&self, place: Place, rvalue: Rvalue) -> Stmt {
        Stmt::new(StmtKind::Assign { place, rvalue }, self.span)
    }

    fn set(&self, local: LocalId, rvalue: Rvalue) -> Stmt {
        self.assign(Place::local(local), rvalue)
    }

    fn block(&self, body: &mut Body, stmts: Vec<Stmt>, terminator: Terminator) -> BasicBlockId {
        body.blocks.push(BasicBlock { stmts, terminator, terminator_span: self.span });
        BasicBlockId(body.blocks.len() as u32 - 1)
    }

    /// A block whose last statement sets `condition`, going to `when_false`
    /// or `when_true`.
    fn branch(
        &self,
        body: &mut Body,
        stmts: Vec<Stmt>,
        condition: LocalId,
        when_false: BasicBlockId,
        when_true: BasicBlockId,
    ) -> BasicBlockId {
        self.block(
            body,
            stmts,
            Terminator::SwitchInt { discr: copy(condition), targets: vec![(0, when_false)], otherwise: when_true },
        )
    }
}

fn copy(local: LocalId) -> Operand {
    Operand::Copy(Place::local(local))
}

fn binary(op: BinOp, lhs: Operand, rhs: Operand) -> Rvalue {
    Rvalue::BinaryOp { op, lhs, rhs }
}

fn numeric(operand: Operand, to: Ty) -> Rvalue {
    Rvalue::Cast { kind: CastKind::Numeric, operand, to }
}

fn int(value: i128, ty: Ty) -> Operand {
    Operand::Const(Const::Int { value: value as u128, ty })
}

/// Statements computing whether an overflow is pending: whether any bit of
/// the masks is set, after moving it down by `flag_shift` for flag words (see
/// `flag_shifts`), or whether a running total's size word (`BlockTotal`) has
/// a bit at or above its limit.
fn pending_overflow(
    body: &mut Body,
    build: &Build,
    common: &CommonTypes,
    masks: &[LocalId],
    flag_shift: &Option<Operand>,
    sizes: &[(LocalId, u32)],
) -> (Vec<Stmt>, LocalId) {
    let word = body.local(masks[0]).ty;
    let mut stmts = Vec::new();
    let mut any = masks[0];
    for mask in &masks[1..] {
        let joined = build.temp(body, word);
        stmts.push(build.set(joined, binary(BinOp::BitOr, copy(any), copy(*mask))));
        any = joined;
    }
    if let Some(shift) = flag_shift {
        let top = build.temp(body, word);
        stmts.push(build.set(top, binary(BinOp::Shr, copy(any), shift.clone())));
        any = top;
    }
    let mut pending = build.temp(body, common.bool_);
    stmts.push(build.set(pending, binary(BinOp::Ne, copy(any), int(0, word))));
    for &(size, limit) in sizes {
        let (high, over, either) = (build.temp(body, common.u64), build.temp(body, common.bool_), build.temp(body, common.bool_));
        stmts.push(build.set(high, binary(BinOp::Shr, copy(size), int(limit as i128, common.u64))));
        stmts.push(build.set(over, binary(BinOp::Ne, copy(high), int(0, common.u64))));
        stmts.push(build.set(either, binary(BinOp::BitOr, copy(pending), copy(over))));
        pending = either;
    }
    (stmts, pending)
}

/// How a group's flag holds its overflows: the shift applied to each
/// overflow word before ORing it in (none: whole words), and to the flag
/// before testing it. With one check whole words are fastest on every C
/// compiler; with several, MSVC and clang differ, so the C compiler decides
/// (`Const::OverflowShift`). Bits groups have masks, not a flag word.
fn flag_shifts(mode: Mode, common: &CommonTypes) -> (Option<Operand>, Option<Operand>) {
    let shift = |per_word| Operand::Const(Const::OverflowShift { per_word });
    match mode {
        Mode::Report => (None, Some(int(63, common.u64))),
        Mode::Rerun => (Some(shift(true)), Some(shift(false))),
        Mode::Bits => (None, None),
    }
}

/// A flag word ORed with an overflow word, moved down by `word_shift` first.
fn accumulate(body: &mut Body, build: &Build, common: &CommonTypes, flag: LocalId, word: LocalId, word_shift: &Option<Operand>) -> Vec<Stmt> {
    let mut stmts = Vec::new();
    let mut bit = word;
    if let Some(shift) = word_shift {
        bit = build.temp(body, common.u64);
        stmts.push(build.set(bit, binary(BinOp::Shr, copy(word), shift.clone())));
    }
    stmts.push(build.set(flag, binary(BinOp::BitOr, copy(flag), copy(bit))));
    stmts
}

/// How a group reports its first overflow.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    /// The loop's one check panics at the group's end.
    Report,
    /// The group re-runs checked.
    Rerun,
    /// A bit per check and iteration.
    Bits,
}

/// Rewrite a vectorisable loop to check its overflows once per group.
///
/// How a group finds its first overflow:
/// - **Report**, when the loop has one check: one flag per group; a set flag
///   panics with that check's message and location, which are those of the
///   first overflow whatever iteration it was. Stores the group made after it
///   are never read: the process aborts (`[SIMD-7]`).
/// - **Re-run**, with several checks: one flag per group. On a set flag every
///   local the loop writes is restored to its value at the group's start and
///   the loop as it was re-runs from there, checked one operation at a time,
///   so it panics at the first overflow exactly. The re-run reads the same
///   inputs: the elements of each view the loop both reads and writes are
///   saved at the group's start (`saves`) and put back first.
/// - **Bits**, where `detect_then_commit` cannot place its loads: one bit per
///   check and iteration; at the group's end the first set bit, by iteration
///   and then by check, reports.
fn group(
    body: &mut Body,
    types: &TypeTable,
    common: &CommonTypes,
    shape: &CountedLoop,
    checks: &[OverflowCheck],
    mode: Mode,
    saves: &[Save],
    totals: &[BlockTotal],
) {
    let flag = mode != Mode::Bits;
    // A group with running totals is the block their run-time proof covers.
    let size = if totals.is_empty() { group_size(mode) } else { TOTAL_BLOCK };
    let (word_shift, flag_shift) = flag_shifts(mode, common);
    let build = Build { span: body.blocks[shape.header].terminator_span };
    let word = |value: u64| int(value as i128, common.usize);
    let header = BasicBlockId(shape.header as u32);

    // The blocks entering the loop from outside, before anything is added.
    let entering: Vec<usize> = (0..body.blocks.len())
        .filter(|b| *b != shape.header && !shape.region.contains(b))
        .filter(|b| successors(&body.blocks[*b].terminator).contains(&header))
        .collect();

    // The in-group position, and the pending-overflow state: one flag, or
    // one mask per check.
    // A flag group ends at `group_end = counter + size`; a bits group
    // counts its position, which is also the bit to set.
    let counter_ty = body.local(shape.counter).ty;
    let position = if flag { build.temp(body, counter_ty) } else { build.temp(body, common.usize) };
    let flags: Vec<LocalId> = if flag {
        vec![build.temp(body, common.u64)]
    } else {
        checks.iter().map(|_| build.temp(body, common.usize)).collect()
    };

    // The report, laid out first so every later block can name it.
    let mut saved: Vec<(LocalId, LocalId)> = Vec::new();
    // One size word per running total: every value it adds, offset so that a
    // small one has no bit at or above the limit, ORed.
    let sizes: Vec<(LocalId, u32)> =
        totals.iter().map(|total| (build.temp(body, common.u64), total.width - 8)).collect();
    // The unsigned copy of each total that has one.
    let copies: Vec<Option<LocalId>> =
        totals.iter().map(|total| total.copied.then(|| build.temp(body, common.u64))).collect();
    // One slot per saved view and group position.
    let slots: Vec<Vec<LocalId>> =
        saves.iter().map(|save| (0..size).map(|_| build.temp(body, save.ty)).collect()).collect();
    let report = if mode == Mode::Rerun {
        saved = saved_locals(body, &build, shape);
        // The group's elements back, from the counter at the group's start
        // (the first saved local), then the locals.
        let first_counter = saved[0].1;
        let mut restore = Vec::new();
        for (save, slots) in saves.iter().zip(&slots) {
            restore.extend(element_copies(body, &build, common, counter_ty, first_counter, save, slots, false));
        }
        restore.extend(saved.iter().map(|(local, save)| build.set(*local, Rvalue::Use(copy(*save)))));
        build.block(body, restore, Terminator::Goto(header))
    } else if mode == Mode::Report {
        let (stmts, pending) = pending_overflow(body, &build, common, &flags, &flag_shift, &sizes);
        let report = BasicBlockId(body.blocks.len() as u32);
        let check = &checks[0];
        body.blocks.push(BasicBlock {
            stmts,
            terminator: Terminator::Assert {
                cond: copy(pending),
                expected: false,
                msg: check.msg.clone(),
                next: BasicBlockId(report.0 + 1),
                span: check.span,
            },
            terminator_span: check.span,
        });
        build.block(body, Vec::new(), Terminator::Unreachable);
        report
    } else {
        let scan = build.temp(body, common.usize);
        let report = BasicBlockId(body.blocks.len() as u32);
        let scan_head = BasicBlockId(report.0 + 1);
        let never = BasicBlockId(report.0 + 2);
        let first_bit = BasicBlockId(report.0 + 3);
        build.block(body, vec![build.set(scan, Rvalue::Use(word(0)))], Terminator::Goto(scan_head));
        let in_range = build.temp(body, common.bool_);
        build.branch(body, vec![build.set(in_range, binary(BinOp::Lt, copy(scan), word(size)))], in_range, never, first_bit);
        build.block(body, Vec::new(), Terminator::Unreachable);
        for (index, check) in checks.iter().enumerate() {
            let (shifted, bit, set) = (build.temp(body, common.usize), build.temp(body, common.usize), build.temp(body, common.bool_));
            let next = BasicBlockId(first_bit.0 + index as u32 + 1);
            body.blocks.push(BasicBlock {
                stmts: vec![
                    build.set(shifted, binary(BinOp::Shr, copy(flags[index]), copy(scan))),
                    build.set(bit, binary(BinOp::BitAnd, copy(shifted), word(1))),
                    build.set(set, binary(BinOp::Ne, copy(bit), word(0))),
                ],
                terminator: Terminator::Assert { cond: copy(set), expected: false, msg: check.msg.clone(), next, span: check.span },
                terminator_span: check.span,
            });
        }
        build.block(body, vec![build.set(scan, binary(BinOp::Add, copy(scan), word(1)))], Terminator::Goto(scan_head));
        report
    };

    // The grouped copy of the body. Its back edge goes to the group head.
    let base = body.blocks.len() as u32;
    let map: HashMap<usize, u32> = shape.region.iter().enumerate().map(|(i, &b)| (b, base + i as u32)).collect();
    let group_head = BasicBlockId(base + shape.region.len() as u32);
    let mut clones: Vec<BasicBlock> = shape.region.iter().map(|&b| body.blocks[b].clone()).collect();
    for clone in &mut clones {
        retarget(&mut clone.terminator, |target| {
            if target == header {
                group_head
            } else {
                map.get(&(target.0 as usize)).map_or(target, |&n| BasicBlockId(n))
            }
        });
    }
    // Each checked `+`/`-` computes its wrapped result and overflow bit in
    // plain arithmetic. Each check's bit goes into the flag, or into its mask
    // at the position.
    // A block's checks are rewritten from its last statement back, so earlier
    // statement indices stay valid.
    let mut order: Vec<usize> = (0..checks.len()).collect();
    order.sort_by_key(|&index| std::cmp::Reverse((checks[index].block, checks[index].stmt)));
    for index in order {
        let check = &checks[index];
        let slot = shape.region.iter().position(|b| *b == check.block).expect("a check is in the loop");
        let Terminator::Assert { cond: Operand::Copy(overflow), next, .. } = clones[slot].terminator.clone() else {
            unreachable!("a check ends in its assert")
        };
        let original = clones[slot].stmts[check.stmt].clone();
        // A running total: its operation in wrapping arithmetic, and the value
        // it adds noted in its size word.
        if let Some(position) = totals.iter().position(|total| total.check == index) {
            let total = &totals[position];
            let (value, offset) = (build.temp(body, common.u64), build.temp(body, common.u64));
            let size = sizes[position].0;
            let mut stmts = Vec::new();
            stmts.push(build.set(value, numeric(total.operand.clone(), common.u64)));
            match copies[position] {
                Some(copy_of_total) => {
                    let StmtKind::CheckedBinaryOp { op, .. } = &original.kind else { unreachable!("a total is a checked operation") };
                    stmts.push(build.set(copy_of_total, binary(*op, copy(copy_of_total), copy(value))));
                }
                None => {
                    let (wrapped, _) =
                        wrapping_form(body, types, common, &build, &original).expect("a total is a `+` or `-` of at most 64 bits");
                    stmts.splice(0..0, wrapped);
                }
            }
            stmts.push(build.set(offset, binary(BinOp::Add, copy(value), int(1i128 << (total.width - 9), common.u64))));
            stmts.push(build.set(size, binary(BinOp::BitOr, copy(size), copy(offset))));
            clones[slot].stmts.splice(check.stmt..=check.stmt, stmts);
            clones[slot].terminator = Terminator::Goto(next);
            continue;
        }
        let (stmts, bit) = overflow_bit(body, types, common, &build, &original, &overflow);
        clones[slot].stmts.splice(check.stmt..=check.stmt, stmts);
        if flag {
            let stmts = accumulate(body, &build, common, flags[0], bit, &word_shift);
            clones[slot].stmts.extend(stmts);
        } else {
            let top = build.temp(body, common.u64);
            clones[slot].stmts.push(build.set(top, binary(BinOp::Shr, copy(bit), int(63, common.u64))));
            let bit_word = build.temp(body, common.usize);
            clones[slot].stmts.push(build.set(bit_word, numeric(copy(top), common.usize)));
            let bit = bit_word;
            let placed = build.temp(body, common.usize);
            clones[slot].stmts.push(build.set(placed, binary(BinOp::Shl, copy(bit), copy(position))));
            clones[slot].stmts.push(build.set(flags[index], binary(BinOp::BitOr, copy(flags[index]), copy(placed))));
        }
        clones[slot].terminator = Terminator::Goto(next);
    }
    // In a bits group the step also advances the position.
    if !flag {
        let step_slot = shape.region.iter().position(|b| *b == shape.step).expect("the step is in the loop");
        clones[step_slot].stmts.push(build.set(position, binary(BinOp::Add, copy(position), word(1))));
    }
    body.blocks.extend(clones);

    // group head: another iteration while the position is inside the group.
    let group_end = BasicBlockId(group_head.0 + 1);
    let next_group = BasicBlockId(group_head.0 + 2);
    let more = build.temp(body, common.bool_);
    let body_entry = BasicBlockId(map[&shape.entry]);
    let inside_group = if flag {
        binary(BinOp::Lt, copy(shape.counter), copy(position))
    } else {
        binary(BinOp::Lt, copy(position), word(size))
    };
    build.branch(body, vec![build.set(more, inside_group)], more, group_end, body_entry);
    // group end: a pending overflow reports; otherwise the next group.
    let (mut stmts, pending) = pending_overflow(body, &build, common, &flags, &flag_shift, &sizes);
    // Each copied total written back (a re-run restores it anyway).
    for (total, copy_of_total) in totals.iter().zip(&copies) {
        if let Some(copy_of_total) = copy_of_total {
            let ty = body.local(total.acc).ty;
            stmts.insert(0, build.set(total.acc, numeric(copy(*copy_of_total), ty)));
        }
    }
    build.branch(body, stmts, pending, next_group, report);
    // start: the group's end (at least a group's worth remains, so it cannot
    // overflow) or position cleared, the flags cleared.
    let clear = if flag {
        let mut clear = vec![
            build.set(position, binary(BinOp::Add, copy(shape.counter), int(size as i128, counter_ty))),
            build.set(flags[0], Rvalue::Use(int(0, common.u64))),
        ];
        for (save, slots) in saves.iter().zip(&slots) {
            clear.extend(element_copies(body, &build, common, counter_ty, shape.counter, save, slots, true));
        }
        clear.extend(sizes.iter().map(|&(size, _)| build.set(size, Rvalue::Use(int(0, common.u64)))));
        for (total, copy_of_total) in totals.iter().zip(&copies) {
            if let Some(copy_of_total) = copy_of_total {
                clear.push(build.set(*copy_of_total, numeric(copy(total.acc), common.u64)));
            }
        }
        clear
    } else {
        let mut clear = vec![build.set(position, Rvalue::Use(word(0)))];
        clear.extend(flags.iter().map(|flag| build.set(*flag, Rvalue::Use(word(0)))));
        clear
    };
    // Each running total small enough at the block's start, or the loop as it
    // was runs the rest.
    let guard = totals_start_test(body, &build, common, totals);
    let entry = group_entry(body, &build, common, shape, size, guard, clear, &saved, group_head);
    debug_assert_eq!(entry, next_group);

    // Nothing in a group that could panic another way or run long is reached
    // with an overflow pending.
    let grouped: Vec<usize> = (base as usize..base as usize + shape.region.len()).collect();
    flush_before(body, &build, common, &grouped, report, &flags, &flag_shift, &sizes);

    for block in entering {
        retarget(&mut body.blocks[block].terminator, |target| if target == header { next_group } else { target });
    }
}

/// Save slots for the counter and for every local the loop writes whose
/// value an iteration reads before writing it (so it carries from one
/// iteration to the next): `(local, slot)`. A store to a view's element is
/// not a write to the local holding the view.
fn saved_locals(body: &mut Body, build: &Build, shape: &CountedLoop) -> Vec<(LocalId, LocalId)> {
    let carried = carried_locals(body, shape);
    let mut locals: Vec<LocalId> = vec![shape.counter];
    let own = |place: &Place| (!place.projection.iter().any(|p| matches!(p, Projection::Index(_)))).then_some(place.local);
    for &block in &shape.region {
        for stmt in &body.blocks[block].stmts {
            let written: Vec<LocalId> = match &stmt.kind {
                StmtKind::Assign { place, .. } => own(place).into_iter().collect(),
                StmtKind::CheckedBinaryOp { dest, overflow, .. } => own(dest).into_iter().chain(own(overflow)).collect(),
                _ => Vec::new(),
            };
            for local in written {
                if carried.contains(&local) && !locals.contains(&local) {
                    locals.push(local);
                }
            }
        }
    }
    locals
        .into_iter()
        .map(|local| {
            let ty = body.local(local).ty;
            (local, build.temp(body, ty))
        })
        .collect()
}

/// The blocks a grouped loop is entered through, laid out from the next free
/// block: `next group` goes on to another group when the loop runs, at least
/// a group's worth (`size` iterations) remains and `guard`'s condition holds,
/// and otherwise to the loop as it was, which finishes the rest; `start` runs
/// `clear`, saves `saved` and goes to `first`. Returns `next group`.
#[allow(clippy::too_many_arguments)]
fn group_entry(
    body: &mut Body,
    build: &Build,
    common: &CommonTypes,
    shape: &CountedLoop,
    size: u64,
    guard: Option<(Vec<Stmt>, LocalId)>,
    mut clear: Vec<Stmt>,
    saved: &[(LocalId, LocalId)],
    first: BasicBlockId,
) -> BasicBlockId {
    let header = BasicBlockId(shape.header as u32);
    let next_group = BasicBlockId(body.blocks.len() as u32);
    let room = BasicBlockId(next_group.0 + 1);
    let guarded = BasicBlockId(next_group.0 + 2);
    let start = BasicBlockId(next_group.0 + if guard.is_some() { 3 } else { 2 });
    let runs = build.temp(body, common.bool_);
    let continues = if shape.inclusive { BinOp::Le } else { BinOp::Lt };
    build.branch(body, vec![build.set(runs, binary(continues, copy(shape.counter), copy(shape.limit)))], runs, header, room);
    let (limit_word, counter_word, remaining, full) =
        (build.temp(body, common.usize), build.temp(body, common.usize), build.temp(body, common.usize), build.temp(body, common.bool_));
    let need = if shape.inclusive { size - 1 } else { size };
    build.branch(
        body,
        vec![
            build.set(limit_word, numeric(copy(shape.limit), common.usize)),
            build.set(counter_word, numeric(copy(shape.counter), common.usize)),
            build.set(remaining, binary(BinOp::Sub, copy(limit_word), copy(counter_word))),
            build.set(full, binary(BinOp::Ge, copy(remaining), int(need as i128, common.usize))),
        ],
        full,
        header,
        if guard.is_some() { guarded } else { start },
    );
    if let Some((stmts, holds)) = guard {
        build.branch(body, stmts, holds, header, start);
    }
    clear.extend(saved.iter().map(|(local, save)| build.set(*save, Rvalue::Use(copy(*local)))));
    let start_id = build.block(body, clear, Terminator::Goto(first));
    debug_assert_eq!(start_id, start);
    next_group
}

/// An access to a written view's element in a block: the statement's
/// position (the terminator's is one past the last statement), the view,
/// whether it needs the element's value (anything but a store of the whole
/// element), and the element's place.
struct ElementAccess {
    position: usize,
    view: usize,
    needs_value: bool,
    element: Place,
}

/// `[SIMD-7]` for a loop that reads a view element it also writes, where
/// `undo_saves` cannot save the group's elements: detect,
/// then commit. Each group runs twice, both runs plain counted loops the C
/// compiler can vectorise.
/// - **Detect** stores nothing. Each written view's element is a local,
///   loaded from memory where the iteration first needs its value; each
///   checked operation computes in wrapping arithmetic and adds its overflow
///   bit to one flag; access checks are left to the commit. Every local the
///   loop carries is then restored to its value at the group's start.
/// - A set flag re-runs the group in the loop as it was, checked one
///   operation at a time, on unchanged memory: it panics at the first
///   overflow exactly.
/// - A clear flag **commits**: the group runs again for real with no
///   overflow check, since nothing in it overflows.
///
/// Returns false, changing nothing, when an access needs the element's value
/// where one path to it has touched the element this iteration and another
/// has not: an element is loaded only where no path could have written it.
fn detect_then_commit(
    body: &mut Body,
    types: &TypeTable,
    common: &CommonTypes,
    shape: &CountedLoop,
    checks: &[OverflowCheck],
    written: &[Place],
) -> bool {
    let mut all = shape.region.clone();
    all.push(shape.header);
    let definitions = definitions(body, &all);
    let dominators = dominators(body, shape.header, &all);
    // The written view a place is an element of, and its index's position.
    let element_of = |body: &Body, block: usize, place: &Place| -> Option<(usize, usize)> {
        let Ok(Some((origin, _))) = vf_place(body, types, &definitions, &dominators, shape.counter, block, place) else {
            return None;
        };
        let view = written.iter().position(|w| *w == origin)?;
        let at = place.projection.iter().position(|p| matches!(p, Projection::Index(_)))?;
        Some((view, at))
    };
    let slot_of = |block: usize| shape.region.iter().position(|b| *b == block);

    // Every block's accesses, in order.
    let mut accesses: HashMap<usize, Vec<ElementAccess>> = HashMap::new();
    for &block in &shape.region {
        let data = &body.blocks[block];
        let mut found = Vec::new();
        for (position, stmt) in data.stmts.iter().enumerate() {
            // Detect drops the access checks.
            if matches!(stmt.kind, StmtKind::BeginAccess { .. } | StmtKind::EndAccess { .. }) {
                continue;
            }
            visit_stmt(stmt, &mut |place: &Place, write: bool, _: bool| {
                if let Some((view, at)) = element_of(body, block, place) {
                    let whole = write
                        && place.projection.len() == at + 1
                        && matches!(&stmt.kind,
                            StmtKind::Assign { place: dest, .. } | StmtKind::CheckedBinaryOp { dest, .. } if dest == place);
                    let element = Place { local: place.local, projection: place.projection[..=at].to_vec() };
                    found.push(ElementAccess { position, view, needs_value: !whole, element });
                }
            });
        }
        visit_terminator(&data.terminator, &mut |place: &Place, _: bool, _: bool| {
            if let Some((view, at)) = element_of(body, block, place) {
                let element = Place { local: place.local, projection: place.projection[..=at].to_vec() };
                found.push(ElementAccess { position: data.stmts.len(), view, needs_value: true, element });
            }
        });
        accesses.insert(block, found);
    }

    // Whether the element's local holds this iteration's value on leaving
    // each block: on every path (`defined`), on some path (`maybe`). An
    // iteration starts with neither.
    let predecessors = predecessors(body);
    let views = written.len();
    let touches = |block: usize, view: usize| accesses[&block].iter().any(|access| access.view == view);
    let entry_state = |out: &HashMap<(usize, usize), (bool, bool)>, block: usize, view: usize| {
        let (mut defined, mut maybe) = (true, false);
        for &from in &predecessors[block] {
            match out.get(&(from, view)) {
                Some(&(d, m)) => {
                    defined &= d;
                    maybe |= m;
                }
                // The loop header: the iteration's start.
                None => defined = false,
            }
        }
        (defined, maybe)
    };
    let mut out: HashMap<(usize, usize), (bool, bool)> =
        shape.region.iter().flat_map(|&b| (0..views).map(move |v| ((b, v), (true, false)))).collect();
    loop {
        let mut changed = false;
        for &block in &shape.region {
            for view in 0..views {
                let (defined, maybe) = entry_state(&out, block, view);
                let touched = touches(block, view);
                let next = (defined || touched, maybe || touched);
                if out[&(block, view)] != next {
                    out.insert((block, view), next);
                    changed = true;
                }
            }
        }
        if !changed {
            break;
        }
    }

    // The loads: before each first access needing the value, where no path
    // has touched the element yet. `(block, position, view, element)`.
    let mut loads: Vec<(usize, usize, usize, Place)> = Vec::new();
    for &block in &shape.region {
        for view in 0..views {
            let (mut defined, mut maybe) = entry_state(&out, block, view);
            let mine: Vec<&ElementAccess> = accesses[&block].iter().filter(|access| access.view == view).collect();
            for (index, access) in mine.iter().enumerate() {
                // A statement's accesses are taken together: it reads its
                // operands before it stores.
                if index > 0 && mine[index - 1].position == access.position {
                    continue;
                }
                let needing = mine[index..].iter().take_while(|a| a.position == access.position).find(|a| a.needs_value);
                if let Some(needing) = needing
                    && !defined
                {
                    if maybe {
                        return false;
                    }
                    loads.push((block, access.position, view, needing.element.clone()));
                }
                defined = true;
                maybe = true;
            }
        }
    }
    // Each view's element type: the local holding it copies it plainly.
    let mut element_tys = Vec::new();
    for view in 0..views {
        let Some(access) = shape.region.iter().flat_map(|b| accesses[b].iter()).find(|a| a.view == view) else {
            return false;
        };
        let ty = place_type(body, types, &access.element);
        if types.needs_drop(ty) {
            return false;
        }
        element_tys.push(ty);
    }

    let build = Build { span: body.blocks[shape.header].terminator_span };
    let header = BasicBlockId(shape.header as u32);
    let entering: Vec<usize> = (0..body.blocks.len())
        .filter(|b| *b != shape.header && slot_of(*b).is_none())
        .filter(|b| successors(&body.blocks[*b].terminator).contains(&header))
        .collect();
    let counter_ty = body.local(shape.counter).ty;
    let (group_end, flag) = (build.temp(body, counter_ty), build.temp(body, common.u64));
    // Several checks: the flag's form is the C compiler's (`flag_shifts`).
    let (word_shift, flag_shift) = flag_shifts(Mode::Rerun, common);
    let elements: Vec<LocalId> = element_tys.iter().map(|ty| build.temp(body, *ty)).collect();
    let saved = saved_locals(body, &build, shape);
    let restore: Vec<Stmt> = saved.iter().map(|(local, save)| build.set(*local, Rvalue::Use(copy(*save)))).collect();

    // Layout: detect's blocks, its head, its end, commit's blocks, its head,
    // then the entry blocks.
    let n = shape.region.len() as u32;
    let detect_base = body.blocks.len() as u32;
    let detect_head = BasicBlockId(detect_base + n);
    let detect_end = BasicBlockId(detect_head.0 + 1);
    let commit_base = detect_end.0 + 1;
    let commit_head = BasicBlockId(commit_base + n);
    let next_group = BasicBlockId(commit_head.0 + 1);
    let entry_slot = slot_of(shape.entry).expect("the entry is in the loop") as u32;
    let clone_all = |base: u32, head: BasicBlockId| -> Vec<BasicBlock> {
        shape
            .region
            .iter()
            .map(|&b| {
                let mut clone = body.blocks[b].clone();
                retarget(&mut clone.terminator, |target| {
                    if target == header {
                        head
                    } else {
                        slot_of(target.0 as usize).map_or(target, |slot| BasicBlockId(base + slot as u32))
                    }
                });
                clone
            })
            .collect()
    };
    let mut detect = clone_all(detect_base, detect_head);
    let mut commit = clone_all(commit_base, commit_head);

    // Detect: every element access is to the element's local.
    for (slot, &block) in shape.region.iter().enumerate() {
        for stmt in &mut detect[slot].stmts {
            if matches!(stmt.kind, StmtKind::BeginAccess { .. } | StmtKind::EndAccess { .. }) {
                stmt.kind = StmtKind::Nop;
            }
        }
        rewrite_places(&mut detect[slot], &|place: &mut Place| {
            if let Some((view, at)) = element_of(body, block, place) {
                *place = Place { local: elements[view], projection: place.projection[at + 1..].to_vec() };
            }
        });
    }
    // Detect: the loads, and each check in wrapping arithmetic into the flag.
    for (slot, &block) in shape.region.iter().enumerate() {
        let old = std::mem::take(&mut detect[slot].stmts);
        let length = old.len();
        let load = |position: usize, stmts: &mut Vec<Stmt>| {
            for (_, _, view, element) in loads.iter().filter(|load| load.0 == block && load.1 == position) {
                stmts.push(build.set(elements[*view], Rvalue::Use(Operand::Copy(element.clone()))));
            }
        };
        let check = checks.iter().find(|check| check.block == block).map(|check| {
            let Terminator::Assert { cond: Operand::Copy(overflow), next, .. } = detect[slot].terminator.clone() else {
                unreachable!("a check ends in its assert")
            };
            (check.stmt, overflow, next)
        });
        let mut stmts = Vec::new();
        let mut bit = None;
        for (position, stmt) in old.into_iter().enumerate() {
            load(position, &mut stmts);
            match &check {
                Some((at, overflow, _)) if *at == position => {
                    let (wrapped, overflowed) = overflow_bit(body, types, common, &build, &stmt, overflow);
                    stmts.extend(wrapped);
                    bit = Some(overflowed);
                }
                _ => stmts.push(stmt),
            }
        }
        load(length, &mut stmts);
        if let (Some((_, _, next)), Some(bit)) = (check, bit) {
            stmts.extend(accumulate(body, &build, common, flag, bit, &word_shift));
            detect[slot].terminator = Terminator::Goto(next);
        }
        detect[slot].stmts = stmts;
    }
    // Commit: each check a plain operation; detect proved none overflows.
    for check in checks {
        let slot = slot_of(check.block).expect("a check is in the loop");
        let stmt = &mut commit[slot].stmts[check.stmt];
        if let StmtKind::CheckedBinaryOp { dest, op: op @ (BinOp::Add | BinOp::Sub | BinOp::Mul), lhs, rhs, .. } = &stmt.kind {
            stmt.kind = StmtKind::Assign { place: dest.clone(), rvalue: binary(*op, lhs.clone(), rhs.clone()) };
        }
        let Terminator::Assert { next, .. } = commit[slot].terminator.clone() else { unreachable!("a check ends in its assert") };
        commit[slot].terminator = Terminator::Goto(next);
    }

    drop_unread_sets(&mut detect, shape.counter);
    body.blocks.extend(detect);
    let more = build.temp(body, common.bool_);
    let head = build.branch(
        body,
        vec![build.set(more, binary(BinOp::Lt, copy(shape.counter), copy(group_end)))],
        more,
        detect_end,
        BasicBlockId(detect_base + entry_slot),
    );
    debug_assert_eq!(head, detect_head);
    // detect end: the locals restored; a set flag re-runs the group checked.
    let (test, pending) = pending_overflow(body, &build, common, &[flag], &flag_shift, &[]);
    build.branch(body, restore.into_iter().chain(test).collect(), pending, commit_head, header);
    body.blocks.extend(commit);
    let more = build.temp(body, common.bool_);
    let head = build.branch(
        body,
        vec![build.set(more, binary(BinOp::Lt, copy(shape.counter), copy(group_end)))],
        more,
        next_group,
        BasicBlockId(commit_base + entry_slot),
    );
    debug_assert_eq!(head, commit_head);
    // start: the group's end (at least a group's worth remains, so it cannot
    // overflow), the flag cleared.
    let clear = vec![
        build.set(group_end, binary(BinOp::Add, copy(shape.counter), int(group_size(Mode::Rerun) as i128, counter_ty))),
        build.set(flag, Rvalue::Use(int(0, common.u64))),
    ];
    let entry = group_entry(body, &build, common, shape, group_size(Mode::Rerun), None, clear, &saved, detect_head);
    debug_assert_eq!(entry, next_group);

    // An inner loop is not entered with an overflow pending: detect's end
    // re-runs the group first.
    let detected: Vec<usize> = (detect_base as usize..(detect_base + n) as usize).collect();
    flush_before(body, &build, common, &detected, detect_end, &[flag], &flag_shift, &[]);

    for block in entering {
        retarget(&mut body.blocks[block].terminator, |target| if target == header { next_group } else { target });
    }
    true
}

/// Remove, until none is left, every assignment of a whole local that
/// nothing in the detect run's `blocks` reads, such as a position whose element
/// became a local. After detect every local is restored or set again before
/// it is read.
fn drop_unread_sets(blocks: &mut [BasicBlock], counter: LocalId) {
    loop {
        let mut read: HashSet<LocalId> = HashSet::from([counter]);
        for block in blocks.iter() {
            let mut note = |place: &Place, written: bool, _: bool| {
                if !written || !place.projection.is_empty() {
                    read.insert(place.local);
                }
                for projection in &place.projection {
                    if let Projection::Index(local) = projection {
                        read.insert(*local);
                    }
                }
            };
            for stmt in &block.stmts {
                match &stmt.kind {
                    StmtKind::Drop { place, .. } => note(place, false, false),
                    _ => visit_stmt(stmt, &mut note),
                }
            }
            visit_terminator(&block.terminator, &mut note);
        }
        let mut removed = false;
        for stmt in blocks.iter_mut().flat_map(|block| block.stmts.iter_mut()) {
            if let StmtKind::Assign { place, rvalue } = &stmt.kind
                && place.projection.is_empty()
                && !read.contains(&place.local)
                && !matches!(rvalue, Rvalue::Ref { .. })
            {
                stmt.kind = StmtKind::Nop;
                removed = true;
            }
        }
        if !removed {
            break;
        }
    }
}

/// A view element range a group saves at its start and puts back before a
/// checked re-run: the view (a local the loop never assigns), the offset of
/// its element from the counter, and the element's type.
struct Save {
    view: Place,
    offset: i128,
    ty: Ty,
}

/// What lets a group with several checks re-run when the loop reads a view
/// it also writes: one `Save` per such view (none when there is none). `None`
/// when such a view is not accessed on every iteration (then the group's
/// elements are not all known to exist), when its element needs a drop, or
/// when the local holding it is assigned in the loop.
fn undo_saves(body: &Body, types: &TypeTable, shape: &CountedLoop, written: &[WrittenView]) -> Option<Vec<Save>> {
    let mut all = shape.region.clone();
    all.push(shape.header);
    let definitions = definitions(body, &all);
    let dominators = dominators(body, shape.header, &all);
    // A block every iteration runs: it dominates the step.
    let every_iteration = |block: usize| dominators.get(&shape.step).is_some_and(|d| d.contains(&block));
    let mut saves = Vec::new();
    for view in written.iter().filter(|view| view.read) {
        // The view as an access every iteration makes names it.
        let mut found: Option<(Place, Place)> = None;
        for &block in shape.region.iter().filter(|&&block| every_iteration(block)) {
            visit_places(&body.blocks[block], &mut |place: &Place, _: bool, access: bool| {
                if access || found.is_some() {
                    return;
                }
                let Ok(Some((origin, _))) =
                    vf_place(body, types, &definitions, &dominators, shape.counter, block, place)
                else {
                    return;
                };
                if origin != view.origin {
                    return;
                }
                if let Some(at) = place.projection.iter().position(|p| matches!(p, Projection::Index(_))) {
                    let view_place = Place { local: place.local, projection: place.projection[..at].to_vec() };
                    let element = Place { local: place.local, projection: place.projection[..=at].to_vec() };
                    found = Some((view_place, element));
                }
            });
        }
        let (view_place, element) = found?;
        let ty = place_type(body, types, &element);
        if types.needs_drop(ty) {
            return None;
        }
        // The local holding the view keeps its value through the loop: only
        // its elements are written.
        let reassigned = shape.region.iter().flat_map(|&b| body.blocks[b].stmts.iter()).any(|stmt| {
            let written: Vec<&Place> = match &stmt.kind {
                StmtKind::Assign { place, .. } => vec![place],
                StmtKind::CheckedBinaryOp { dest, overflow, .. } => vec![dest, overflow],
                _ => Vec::new(),
            };
            written.iter().any(|place| {
                place.local == view_place.local && !place.projection.iter().any(|p| matches!(p, Projection::Index(_)))
            })
        });
        if reassigned {
            return None;
        }
        saves.push(Save { view: view_place, offset: view.offset, ty });
    }
    Some(saves)
}

/// Copies between a saved view's group elements and their slots, the group
/// starting at `counter`: into the slots (`into_slots`) at the group's start,
/// back into the view before a re-run.
#[allow(clippy::too_many_arguments)]
fn element_copies(
    body: &mut Body,
    build: &Build,
    common: &CommonTypes,
    counter_ty: Ty,
    counter: LocalId,
    save: &Save,
    slots: &[LocalId],
    into_slots: bool,
) -> Vec<Stmt> {
    let (first, base) = (build.temp(body, counter_ty), build.temp(body, common.usize));
    let mut stmts = vec![
        build.set(first, binary(BinOp::Add, copy(counter), int(save.offset, counter_ty))),
        build.set(base, numeric(copy(first), common.usize)),
    ];
    for (position, slot) in slots.iter().enumerate() {
        let index = build.temp(body, common.usize);
        stmts.push(build.set(index, binary(BinOp::Add, copy(base), int(position as i128, common.usize))));
        let mut element = save.view.clone();
        element.projection.push(Projection::Index(index));
        stmts.push(if into_slots {
            build.set(*slot, Rvalue::Use(Operand::Copy(element)))
        } else {
            build.assign(element, Rvalue::Use(copy(*slot)))
        });
    }
    stmts
}

/// Iterations in a block whose running totals are proved safe at run time.
const TOTAL_BLOCK: u64 = 64;

/// A running total proved safe block by block (`[SIMD-7]`, ODR-086): the check
/// (its index among the loop's), the total, its width `w`, and the value each
/// operation adds or subtracts. Before a block the total must lie in
/// [−2^(w−2), 2^(w−2)), and every value the block adds in [−2^(w−9),
/// 2^(w−9)): 64 of them move the total by at most 2^(w−3), so no partial sum
/// leaves the type in any order.
///
/// When nothing else in the loop reads the total, a block adds into an
/// unsigned copy of it, written back at the block's end: MSVC recognises only
/// that form as a sum it can vectorise (reason 1105 otherwise).
struct BlockTotal {
    check: usize,
    acc: LocalId,
    width: u32,
    operand: Operand,
    copied: bool,
}

/// The running totals of a loop's checks, each proved safe block by block;
/// `None` when a total cannot be: not signed, narrower than 16 bits, written
/// anywhere else in the loop, or adding something that names the total.
/// `[SIMD-5]` (ODR-088) — whether the loop body divides integers in a way no
/// vector instruction set can: by a divisor that is not a constant (no SIMD
/// integer division exists, on x86 or ARM), or, at 64 bits or more, by a
/// constant that is not a power of two (no 64-bit vector multiply-high to
/// divide by it). Such a loop runs one iteration at a time whatever its
/// checks, so it is not in vectorisable form and its checks are not grouped.
pub(crate) fn scalar_only(body: &Body, types: &TypeTable, shape: &CountedLoop) -> bool {
    let divides = |op: &BinOp, rhs: &Operand, ty: Ty| {
        if !matches!(op, BinOp::Div | BinOp::Rem | BinOp::FloorDiv | BinOp::FloorRem) || types.is_float(ty) {
            return false;
        }
        let wide = ember_types::bit_width(types, ty).is_some_and(|w| w >= 64);
        match rhs {
            Operand::Const(Const::Int { value, .. }) => wide && !(value.is_power_of_two()),
            _ => true,
        }
    };
    shape.region.iter().any(|&block| {
        body.blocks[block].stmts.iter().any(|stmt| match &stmt.kind {
            StmtKind::Assign { place, rvalue: Rvalue::BinaryOp { op, rhs, .. } } => {
                divides(op, rhs, place_type(body, types, place))
            }
            StmtKind::CheckedBinaryOp { dest, op, rhs, .. } => divides(op, rhs, place_type(body, types, dest)),
            _ => false,
        })
    })
}

fn block_totals(body: &Body, types: &TypeTable, shape: &CountedLoop, checks: &[OverflowCheck]) -> Option<Vec<BlockTotal>> {
    let mut totals = Vec::new();
    for (index, check) in checks.iter().enumerate() {
        let stmt = &body.blocks[check.block].stmts[check.stmt];
        if !running_total(stmt) {
            continue;
        }
        let StmtKind::CheckedBinaryOp { dest, op, lhs, rhs, .. } = &stmt.kind else { return None };
        let acc = dest.local;
        let is_acc = |operand: &Operand| matches!(operand, Operand::Copy(place) if *place == Place::local(acc));
        let operand = match op {
            BinOp::Add if is_acc(lhs) => rhs,
            BinOp::Add if is_acc(rhs) => lhs,
            BinOp::Sub if is_acc(lhs) => rhs,
            _ => return None,
        };
        if matches!(operand, Operand::Copy(place) | Operand::Move(place) if place.local == acc
            || place.projection.iter().any(|p| matches!(p, Projection::Index(local) if *local == acc)))
        {
            return None;
        }
        let ty = body.local(acc).ty;
        let width = ember_types::bit_width(types, ty)?;
        if ember_types::is_signed(types, ty) != Some(true) || !(16..=64).contains(&width) {
            return None;
        }
        let writes = shape
            .region
            .iter()
            .flat_map(|&b| body.blocks[b].stmts.iter())
            .filter(|stmt| match &stmt.kind {
                StmtKind::Assign { place, .. } => place.local == acc,
                StmtKind::CheckedBinaryOp { dest, overflow, .. } => dest.local == acc || overflow.local == acc,
                _ => false,
            })
            .count();
        if writes != 1 {
            return None;
        }
        // Read nowhere but by its own operation.
        let mentions = |named: &mut dyn FnMut(&Place)| {
            for &b in &shape.region {
                let data = &body.blocks[b];
                for stmt in &data.stmts {
                    if !std::ptr::eq(stmt, &body.blocks[check.block].stmts[check.stmt]) {
                        visit_stmt(stmt, &mut |place: &Place, _: bool, _: bool| named(place));
                    }
                }
                visit_terminator(&data.terminator, &mut |place: &Place, _: bool, _: bool| named(place));
            }
        };
        let mut read_elsewhere = false;
        mentions(&mut |place: &Place| {
            read_elsewhere |= place.local == acc
                || place.projection.iter().any(|p| matches!(p, Projection::Index(local) if *local == acc));
        });
        totals.push(BlockTotal { check: index, acc, width: width as u32, operand: operand.clone(), copied: !read_elsewhere });
    }
    Some(totals)
}

/// The test before a block of running totals: every total in [−2^(w−2),
/// 2^(w−2)), tested as the total plus 2^(w−2) having no bit at or above
/// w − 1. `None` when there are no totals.
fn totals_start_test(body: &mut Body, build: &Build, common: &CommonTypes, totals: &[BlockTotal]) -> Option<(Vec<Stmt>, LocalId)> {
    let mut stmts = Vec::new();
    let mut holds: Option<LocalId> = None;
    for total in totals {
        let (value, offset, high, small) =
            (build.temp(body, common.u64), build.temp(body, common.u64), build.temp(body, common.u64), build.temp(body, common.bool_));
        stmts.push(build.set(value, numeric(copy(total.acc), common.u64)));
        stmts.push(build.set(offset, binary(BinOp::Add, copy(value), int(1i128 << (total.width - 2), common.u64))));
        stmts.push(build.set(high, binary(BinOp::Shr, copy(offset), int(total.width as i128 - 1, common.u64))));
        stmts.push(build.set(small, binary(BinOp::Eq, copy(high), int(0, common.u64))));
        holds = Some(match holds {
            None => small,
            Some(earlier) => {
                let both = build.temp(body, common.bool_);
                stmts.push(build.set(both, binary(BinOp::BitAnd, copy(earlier), copy(small))));
                both
            }
        });
    }
    holds.map(|holds| (stmts, holds))
}

/// Whether a checked operation is a running total: its result is one of its
/// operands.
fn running_total(stmt: &Stmt) -> bool {
    matches!(&stmt.kind,
        StmtKind::CheckedBinaryOp { dest, lhs, rhs, .. }
            if dest.projection.is_empty()
                && [lhs, rhs].iter().any(|operand| matches!(operand, Operand::Copy(place) if *place == *dest)))
}

/// Send every edge entering the loop from outside to `entry` instead.
fn enter_through(body: &mut Body, shape: &CountedLoop, entry: BasicBlockId) {
    let header = BasicBlockId(shape.header as u32);
    let first_new = entry.0 as usize;
    for block in 0..first_new.min(body.blocks.len()) {
        if block == shape.header || shape.region.contains(&block) {
            continue;
        }
        retarget(&mut body.blocks[block].terminator, |target| if target == header { entry } else { target });
    }
}

/// Before every access check and every inner loop header in the grouped
/// blocks, test the masks: a pending overflow is reported first.
fn flush_before(
    body: &mut Body,
    build: &Build,
    common: &CommonTypes,
    grouped: &[usize],
    report: BasicBlockId,
    masks: &[LocalId],
    flag_shift: &Option<Operand>,
    sizes: &[(LocalId, u32)],
) {
    // Inner loop headers, found before any block is split.
    let inside: HashSet<usize> = grouped.iter().copied().collect();
    let predecessors = predecessors(body);
    // A block reached again from itself without leaving the group's blocks
    // (the group's own back edge leaves them, through the group head).
    let reach_within = |start: usize| {
        let mut seen = HashSet::new();
        let mut work: Vec<usize> = successors(&body.blocks[start].terminator).into_iter().map(|b| b.0 as usize).collect();
        while let Some(block) = work.pop() {
            if !inside.contains(&block) || !seen.insert(block) {
                continue;
            }
            work.extend(successors(&body.blocks[block].terminator).into_iter().map(|b| b.0 as usize));
        }
        seen
    };
    let headers: Vec<usize> = grouped
        .iter()
        .copied()
        .filter(|&b| {
            let reach = reach_within(b);
            predecessors[b].iter().any(|p| inside.contains(p) && reach.contains(p))
        })
        .collect();

    // Access checks: split each block before each check.
    let mut work: Vec<usize> = grouped.to_vec();
    let mut all_grouped: Vec<usize> = grouped.to_vec();
    while let Some(block) = work.pop() {
        let Some(at) = body.blocks[block].stmts.iter().position(|stmt| matches!(stmt.kind, StmtKind::BeginAccess { .. })) else {
            continue;
        };
        let mut rest = body.blocks[block].stmts.split_off(at);
        let tail = rest.split_off(2);
        let terminator = std::mem::replace(&mut body.blocks[block].terminator, Terminator::Unreachable);
        let span = body.blocks[block].terminator_span;
        // After the check: the remaining statements and the terminator.
        body.blocks.push(BasicBlock { stmts: tail, terminator, terminator_span: span });
        let after = BasicBlockId(body.blocks.len() as u32 - 1);
        // The check itself.
        body.blocks.push(BasicBlock { stmts: rest, terminator: Terminator::Goto(after), terminator_span: span });
        let check = BasicBlockId(body.blocks.len() as u32 - 1);
        let (stmts, pending) = pending_overflow(body, build, common, masks, flag_shift, sizes);
        body.blocks[block].stmts.extend(stmts);
        body.blocks[block].terminator =
            Terminator::SwitchInt { discr: copy(pending), targets: vec![(0, check)], otherwise: report };
        work.push(after.0 as usize);
        all_grouped.push(after.0 as usize);
        all_grouped.push(check.0 as usize);
    }

    // Inner loops: a test on every edge into their header.
    for inner in headers {
        let (stmts, pending) = pending_overflow(body, build, common, masks, flag_shift, sizes);
        let flush = build.branch(body, stmts, pending, BasicBlockId(inner as u32), report);
        for &block in &all_grouped {
            retarget(&mut body.blocks[block].terminator, |t| if t.0 as usize == inner { flush } else { t });
        }
    }
}

/// A checked operation in a group: its replacement statements and its
/// overflow word, a `u64` whose top bit is set when it overflowed. A `+` or
/// `-` of at most 64 bits becomes plain arithmetic (`wrapping_form`);
/// anything else stays checked, its flag moved to the top bit.
fn overflow_bit(
    body: &mut Body,
    types: &TypeTable,
    common: &CommonTypes,
    build: &Build,
    stmt: &Stmt,
    overflow: &Place,
) -> (Vec<Stmt>, LocalId) {
    if let Some(wrapped) = wrapping_form(body, types, common, build, stmt) {
        return wrapped;
    }
    let (flag, word) = (build.temp(body, common.u64), build.temp(body, common.u64));
    let stmts = vec![
        stmt.clone(),
        build.set(flag, numeric(Operand::Copy(overflow.clone()), common.u64)),
        build.set(word, binary(BinOp::Shl, copy(flag), int(63, common.u64))),
    ];
    (stmts, word)
}

/// A checked `+` or `-` rewritten to its wrapped result and its overflow word
/// (a `u64` whose top bit is set when it overflowed; the other bits mean
/// nothing) in plain arithmetic with no comparison, which the C compiler can
/// vectorise: MSVC vectorises no loop that turns a comparison into a value.
/// A group ORs the words and tests the top bit once, after the loop; moving
/// each bit down first cost MSVC 11% on a list loop. `None` keeps the checked
/// operation (a multiply, a shift, a 128-bit type).
fn wrapping_form(
    body: &mut Body,
    types: &TypeTable,
    common: &CommonTypes,
    build: &Build,
    stmt: &Stmt,
) -> Option<(Vec<Stmt>, LocalId)> {
    let StmtKind::CheckedBinaryOp { dest, op, lhs, rhs, .. } = &stmt.kind else { return None };
    if !matches!(op, BinOp::Add | BinOp::Sub) {
        return None;
    }
    let ty = place_type(body, types, dest);
    let width = ember_types::bit_width(types, ty)?;
    if width > 64 {
        return None;
    }
    let signed = ember_types::is_signed(types, ty)?;
    let word = common.u64;
    let at = |place: Place, rvalue: Rvalue| Stmt::new(StmtKind::Assign { place, rvalue }, stmt.span);
    let mut out = Vec::new();
    let mut temp = |body: &mut Body, rvalue: Rvalue, ty: Ty| {
        let local = build.temp(body, ty);
        out.push(at(Place::local(local), rvalue));
        local
    };
    let not = |local: LocalId| Rvalue::UnaryOp { op: UnOp::BitNot, operand: copy(local) };
    let (result, overflow_word) = if width == 64 {
        // The operation wraps through the unsigned type.
        let a = temp(body, numeric(lhs.clone(), word), word);
        let b = temp(body, numeric(rhs.clone(), word), word);
        let sum = temp(body, binary(*op, copy(a), copy(b)), word);
        let result = temp(body, numeric(copy(sum), ty), ty);
        // The top bit of the overflow word: for a signed `+`, both operands'
        // signs differ from the result's; for a signed `-`, the operands'
        // signs differ and the result's differs from the first's. For an
        // unsigned `+` the carry out, for an unsigned `-` the borrow out.
        let overflow = match (signed, op) {
            (true, BinOp::Add) => {
                let x = temp(body, binary(BinOp::BitXor, copy(a), copy(sum)), word);
                let y = temp(body, binary(BinOp::BitXor, copy(b), copy(sum)), word);
                temp(body, binary(BinOp::BitAnd, copy(x), copy(y)), word)
            }
            (true, _) => {
                let x = temp(body, binary(BinOp::BitXor, copy(a), copy(b)), word);
                let y = temp(body, binary(BinOp::BitXor, copy(a), copy(sum)), word);
                temp(body, binary(BinOp::BitAnd, copy(x), copy(y)), word)
            }
            (false, BinOp::Add) => {
                let both = temp(body, binary(BinOp::BitAnd, copy(a), copy(b)), word);
                let either = temp(body, binary(BinOp::BitOr, copy(a), copy(b)), word);
                let lost = temp(body, not(sum), word);
                let carried = temp(body, binary(BinOp::BitAnd, copy(either), copy(lost)), word);
                temp(body, binary(BinOp::BitOr, copy(both), copy(carried)), word)
            }
            (false, _) => {
                let not_a = temp(body, not(a), word);
                let borrowed = temp(body, binary(BinOp::BitAnd, copy(not_a), copy(b)), word);
                let either = temp(body, binary(BinOp::BitOr, copy(not_a), copy(b)), word);
                let kept = temp(body, binary(BinOp::BitAnd, copy(either), copy(sum)), word);
                temp(body, binary(BinOp::BitOr, copy(borrowed), copy(kept)), word)
            }
        };
        (result, overflow)
    } else if signed {
        // Narrow signed: exact in 64 bits; it overflowed when narrowing and
        // widening back changes it.
        let i64_ = common.i64;
        let a = temp(body, numeric(lhs.clone(), i64_), i64_);
        let b = temp(body, numeric(rhs.clone(), i64_), i64_);
        let wide = temp(body, binary(*op, copy(a), copy(b)), i64_);
        let result = temp(body, numeric(copy(wide), ty), ty);
        let back = temp(body, numeric(copy(result), i64_), i64_);
        let changed = temp(body, binary(BinOp::BitXor, copy(back), copy(wide)), i64_);
        let changed = temp(body, numeric(copy(changed), word), word);
        // Nonzero sets the top bit of `x | -x`.
        let negated = temp(body, binary(BinOp::Sub, int(0, word), copy(changed)), word);
        (result, temp(body, binary(BinOp::BitOr, copy(changed), copy(negated)), word))
    } else {
        // Narrow unsigned: exact in 64 bits; a `+` carries into bit `width`,
        // moved to the top; a `-` that borrows wraps to the top of the range.
        let a = temp(body, numeric(lhs.clone(), word), word);
        let b = temp(body, numeric(rhs.clone(), word), word);
        let wide = temp(body, binary(*op, copy(a), copy(b)), word);
        let result = temp(body, numeric(copy(wide), ty), ty);
        let overflow = if *op == BinOp::Add {
            temp(body, binary(BinOp::Shl, copy(wide), int(63 - width as i128, word)), word)
        } else {
            wide
        };
        (result, overflow)
    };
    out.push(at(dest.clone(), Rvalue::Use(copy(result))));
    Some((out, overflow_word))
}

/// `[SIMD-7]`'s running totals: when every overflow check of the loop is a
/// running total `acc = acc ± x`, `x` an integer of at most 32 bits widened
/// into a 64-bit signed `acc` written nowhere else in the loop, a copy of the
/// loop with no check at all runs when the trip count is at most 2³¹ and every
/// total starts within `2⁶³ − 1 − 2³¹·m` of zero (`m` its elements' largest
/// magnitude): no partial sum can overflow in any grouping. Returns the test's
/// first block, which falls back to `otherwise`.
fn unchecked_totals(
    body: &mut Body,
    types: &TypeTable,
    common: &CommonTypes,
    shape: &CountedLoop,
    checks: &[OverflowCheck],
    otherwise: BasicBlockId,
) -> Option<BasicBlockId> {
    let mut all = shape.region.clone();
    all.push(shape.header);
    let definitions = definitions(body, &all);
    let mut totals: Vec<(LocalId, i128)> = Vec::new();
    for check in checks {
        let StmtKind::CheckedBinaryOp { dest, op, lhs, rhs, .. } = &body.blocks[check.block].stmts[check.stmt].kind else {
            return None;
        };
        if !matches!(op, BinOp::Add | BinOp::Sub) || !dest.projection.is_empty() {
            return None;
        }
        let acc = dest.local;
        let acc_ty = body.local(acc).ty;
        if ember_types::bit_width(types, acc_ty) != Some(64) || ember_types::is_signed(types, acc_ty) != Some(true) {
            return None;
        }
        let element = match (lhs, rhs) {
            (Operand::Copy(l), Operand::Copy(r)) if *l == Place::local(acc) && r.projection.is_empty() => r.local,
            (Operand::Copy(l), Operand::Copy(r)) if *op == BinOp::Add && *r == Place::local(acc) && l.projection.is_empty() => l.local,
            _ => return None,
        };
        let (_, Definition::Value(Rvalue::Cast { kind: CastKind::Numeric | CastKind::Widen, operand: Operand::Copy(source), .. })) =
            definitions.get(&element)?
        else {
            return None;
        };
        let source_ty = place_type(body, types, source);
        let bits = ember_types::bit_width(types, source_ty)?;
        if bits > 32 {
            return None;
        }
        let magnitude: i128 = if ember_types::is_signed(types, source_ty)? { 1i128 << (bits - 1) } else { (1i128 << bits) - 1 };
        let writes = all
            .iter()
            .flat_map(|&b| body.blocks[b].stmts.iter())
            .filter(|stmt| match &stmt.kind {
                StmtKind::Assign { place, .. } => place.local == acc,
                StmtKind::CheckedBinaryOp { dest, .. } => dest.local == acc,
                _ => false,
            })
            .count();
        if writes != 1 || totals.iter().any(|(local, _)| *local == acc) {
            return None;
        }
        totals.push((acc, magnitude));
    }
    let build = Build { span: body.blocks[shape.header].terminator_span };
    let header = BasicBlockId(shape.header as u32);

    // The unchecked copy: the header and the body, each check a plain
    // operation the test has proved cannot overflow.
    let base = body.blocks.len() as u32;
    let mut copied = shape.region.clone();
    copied.push(shape.header);
    let map: HashMap<usize, u32> = copied.iter().enumerate().map(|(i, &b)| (b, base + i as u32)).collect();
    let mut clones: Vec<BasicBlock> = copied.iter().map(|&b| body.blocks[b].clone()).collect();
    for clone in &mut clones {
        retarget(&mut clone.terminator, |target| map.get(&(target.0 as usize)).map_or(target, |&n| BasicBlockId(n)));
    }
    for check in checks {
        let slot = copied.iter().position(|b| *b == check.block).expect("a check is in the loop");
        let StmtKind::CheckedBinaryOp { dest, op, lhs, rhs, .. } = clones[slot].stmts[check.stmt].kind.clone() else {
            unreachable!("a check is a checked operation")
        };
        let span = clones[slot].stmts[check.stmt].span;
        clones[slot].stmts[check.stmt] = Stmt::new(StmtKind::Assign { place: dest, rvalue: binary(op, lhs, rhs) }, span);
        let Terminator::Assert { next, .. } = clones[slot].terminator.clone() else { unreachable!("a check ends in its assert") };
        clones[slot].terminator = Terminator::Goto(next);
    }
    body.blocks.extend(clones);
    let unchecked_header = BasicBlockId(map[&shape.header]);

    // The test, one condition per block.
    let entry = BasicBlockId(body.blocks.len() as u32);
    let runs = build.temp(body, common.bool_);
    let continues = if shape.inclusive { BinOp::Le } else { BinOp::Lt };
    let (limit_word, counter_word, trips, short) =
        (build.temp(body, common.usize), build.temp(body, common.usize), build.temp(body, common.usize), build.temp(body, common.bool_));
    let max_steps = if shape.inclusive { TOTAL_TRIPS - 1 } else { TOTAL_TRIPS };
    build.branch(body, vec![build.set(runs, binary(continues, copy(shape.counter), copy(shape.limit)))], runs, header, BasicBlockId(entry.0 + 1));
    let mut next = BasicBlockId(entry.0 + 2);
    build.branch(
        body,
        vec![
            build.set(limit_word, numeric(copy(shape.limit), common.usize)),
            build.set(counter_word, numeric(copy(shape.counter), common.usize)),
            build.set(trips, binary(BinOp::Sub, copy(limit_word), copy(counter_word))),
            build.set(short, binary(BinOp::Le, copy(trips), int(max_steps, common.usize))),
        ],
        short,
        otherwise,
        next,
    );
    for (index, (acc, magnitude)) in totals.iter().enumerate() {
        let reach = i64::MAX as i128 - TOTAL_TRIPS * magnitude;
        let acc_ty = body.local(*acc).ty;
        let (low, high) = (build.temp(body, common.bool_), build.temp(body, common.bool_));
        let after = if index + 1 == totals.len() { unchecked_header } else { BasicBlockId(next.0 + 2) };
        build.branch(body, vec![build.set(low, binary(BinOp::Ge, copy(*acc), int(-reach, acc_ty)))], low, otherwise, BasicBlockId(next.0 + 1));
        build.branch(body, vec![build.set(high, binary(BinOp::Le, copy(*acc), int(reach, acc_ty)))], high, otherwise, after);
        next = BasicBlockId(next.0 + 2);
    }
    Some(entry)
}

/// The locals live at the loop header over one iteration: read before they
/// are written, starting from the header, the back edge not followed.
fn carried_locals(body: &Body, shape: &CountedLoop) -> HashSet<LocalId> {
    fn place_uses(place: &Place, uses: &mut Vec<LocalId>) {
        uses.push(place.local);
        for projection in &place.projection {
            if let Projection::Index(local) = projection {
                uses.push(*local);
            }
        }
    }
    // A write to a whole local defines it; a write into part of it uses it.
    fn written(place: &Place, uses: &mut Vec<LocalId>, defs: &mut Vec<LocalId>) {
        if place.projection.is_empty() {
            defs.push(place.local);
        } else {
            place_uses(place, uses);
        }
    }
    fn operand(operand: &Operand, uses: &mut Vec<LocalId>) {
        if let Operand::Copy(place) | Operand::Move(place) = operand {
            place_uses(place, uses);
        }
    }
    let step = |live: &mut HashSet<LocalId>, uses: Vec<LocalId>, defs: Vec<LocalId>| {
        for def in defs {
            live.remove(&def);
        }
        live.extend(uses);
    };
    let mut blocks = shape.region.clone();
    blocks.push(shape.header);
    let inside: HashSet<usize> = blocks.iter().copied().collect();
    let mut live_in: HashMap<usize, HashSet<LocalId>> = blocks.iter().map(|&b| (b, HashSet::new())).collect();
    loop {
        let mut changed = false;
        for &block in blocks.iter().rev() {
            let data = &body.blocks[block];
            let mut live: HashSet<LocalId> = HashSet::new();
            for successor in successors(&data.terminator) {
                let successor = successor.0 as usize;
                if successor != shape.header && inside.contains(&successor) {
                    live.extend(live_in[&successor].iter().copied());
                }
            }
            let (mut uses, mut defs) = (Vec::new(), Vec::new());
            match &data.terminator {
                Terminator::SwitchInt { discr, .. } => operand(discr, &mut uses),
                Terminator::Assert { cond, msg, .. } => {
                    operand(cond, &mut uses);
                    if let AssertKind::Bounds { len, index } = msg {
                        operand(len, &mut uses);
                        operand(index, &mut uses);
                    }
                }
                Terminator::Call { args, dest, func, .. } => {
                    written(dest, &mut uses, &mut defs);
                    for arg in args {
                        operand(arg, &mut uses);
                    }
                    if let FuncRef::Indirect { operand: callee, .. } = func {
                        operand(callee, &mut uses);
                    }
                }
                Terminator::Goto(_) | Terminator::Return | Terminator::Unreachable => {}
            }
            step(&mut live, uses, defs);
            for stmt in data.stmts.iter().rev() {
                let (mut uses, mut defs) = (Vec::new(), Vec::new());
                match &stmt.kind {
                    StmtKind::Assign { place, rvalue } => {
                        written(place, &mut uses, &mut defs);
                        match rvalue {
                            Rvalue::Use(value) | Rvalue::UnaryOp { operand: value, .. } | Rvalue::Cast { operand: value, .. } => {
                                operand(value, &mut uses)
                            }
                            Rvalue::BinaryOp { lhs, rhs, .. } => {
                                operand(lhs, &mut uses);
                                operand(rhs, &mut uses);
                            }
                            Rvalue::Aggregate { operands, .. } => {
                                for value in operands {
                                    operand(value, &mut uses);
                                }
                            }
                            Rvalue::Repeat { value, .. } => operand(value, &mut uses),
                            Rvalue::Discriminant(place) | Rvalue::Ref { place, .. } => place_uses(place, &mut uses),
                        }
                    }
                    StmtKind::CheckedBinaryOp { dest, overflow, lhs, rhs, .. } => {
                        written(dest, &mut uses, &mut defs);
                        written(overflow, &mut uses, &mut defs);
                        operand(lhs, &mut uses);
                        operand(rhs, &mut uses);
                    }
                    StmtKind::Drop { place, .. } => place_uses(place, &mut uses),
                    StmtKind::StorageLive(local) => defs.push(*local),
                    StmtKind::BeginAccess { place, .. }
                    | StmtKind::BeginAccessTransfer { place, .. }
                    | StmtKind::EndAccess { place, .. }
                    | StmtKind::EndAccessTransfer { place, .. } => place_uses(place, &mut uses),
                    StmtKind::StorageDead(_) | StmtKind::Nop => {}
                }
                step(&mut live, uses, defs);
            }
            if live != live_in[&block] {
                live_in.insert(block, live);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    live_in.remove(&shape.header).unwrap_or_default()
}

// ---------------------------------------------------------------------------
// Loop-invariant views.

/// Hoist, out of every counted loop, the pure work each turn begins with
/// whose inputs the loop cannot change: the view a loop inside it iterates
/// (`&list`, the view of it, its iterator's fields, its length), when nothing
/// the loop writes can change the list's header. A loop whose turn is then
/// only its inner loop is a perfect nest, which MSVC reorders as it does the
/// C twin's (the particles benchmark: 1.5x C, and C's speed with it); every
/// compiler is spared rebuilding the view each turn. Returns how many
/// computations moved.
pub fn hoist_invariant_views_all(bodies: &mut [Body], types: &TypeTable) -> usize {
    let summaries = Summaries::compute(bodies, types);
    bodies.iter_mut().map(|body| hoist_views(body, types, &summaries)).sum()
}

fn hoist_views(body: &mut Body, types: &TypeTable, summaries: &Summaries) -> usize {
    // A view hoisted out of an inner loop stands at the start of the outer
    // loop's turn, which a later round hoists on.
    let mut total = 0;
    for _ in 0..8 {
        let mut moved = 0;
        for header in 0..body.blocks.len() {
            if let Some(shape) = counted_loop(body, types, header) {
                moved += hoist_from(body, types, summaries, &shape);
            }
        }
        total += moved;
        if moved == 0 {
            break;
        }
    }
    total
}

/// One computation that moves: a statement, or a block's built-in call.
#[derive(Clone, Copy, PartialEq)]
enum Hoisted {
    Stmt(usize, usize),
    Call(usize),
}

fn deref_of(place: &Place) -> Place {
    let mut projection = place.projection.clone();
    projection.push(Projection::Deref);
    Place { local: place.local, projection }
}

fn hoist_from(body: &mut Body, types: &TypeTable, summaries: &Summaries, shape: &CountedLoop) -> usize {
    let facts = BodyFacts::new(body, types);
    let mut inside = shape.region.clone();
    inside.push(shape.header);
    let inside_set: HashSet<usize> = inside.iter().copied().collect();
    let written = loop_writes(body, types, summaries, &facts, &inside);
    // Values each local is given in the loop, and the locals whose storage
    // begins or ends in it.
    let mut writes: HashMap<LocalId, usize> = HashMap::new();
    let mut scoped: HashSet<LocalId> = HashSet::new();
    for &block in &inside {
        for stmt in &body.blocks[block].stmts {
            match &stmt.kind {
                StmtKind::Assign { place, .. } => *writes.entry(place.local).or_default() += 1,
                StmtKind::CheckedBinaryOp { dest, overflow, .. } => {
                    *writes.entry(dest.local).or_default() += 1;
                    *writes.entry(overflow.local).or_default() += 1;
                }
                StmtKind::Drop { place, .. } => *writes.entry(place.local).or_default() += 1,
                StmtKind::StorageLive(local) | StmtKind::StorageDead(local) => {
                    scoped.insert(*local);
                }
                _ => {}
            }
        }
        if let Terminator::Call { dest, .. } = &body.blocks[block].terminator {
            *writes.entry(dest.local).or_default() += 1;
        }
    }
    // What is read outside the loop.
    let mut outside: HashSet<LocalId> = HashSet::new();
    for (index, block) in body.blocks.iter().enumerate() {
        if inside_set.contains(&index) {
            continue;
        }
        visit_places(block, &mut |place: &Place, written: bool, _| {
            if !written || !place.projection.is_empty() {
                outside.insert(place.local);
            }
        });
    }
    let predecessors = predecessors(body);
    // A computation from locals alone the C compiler moves itself. Moved
    // here, it would stand in an outer loop's turn before this loop, and a
    // nest with work between its loops is not perfect: MSVC reorders only a
    // perfect nest (`enumerate(start=round)`, 1.5x C). It moves only when
    // it leaves every loop around this one, so when nothing on a cycle
    // through this loop's header changes its inputs.
    let forward: HashSet<usize> = reach_order(body, shape.header, usize::MAX).into_iter().collect();
    let mut cycle: HashSet<usize> = HashSet::new();
    let mut work = vec![shape.header];
    while let Some(block) = work.pop() {
        for &from in &predecessors[block] {
            if forward.contains(&from) && cycle.insert(from) {
                work.push(from);
            }
        }
    }
    let mut cycle_writes: HashSet<LocalId> = HashSet::new();
    for &block in &cycle {
        visit_places(&body.blocks[block], &mut |place: &Place, written: bool, _| {
            if written {
                cycle_writes.insert(place.local);
            }
        });
        for stmt in &body.blocks[block].stmts {
            if let StmtKind::StorageLive(local) | StmtKind::StorageDead(local) = stmt.kind {
                cycle_writes.insert(local);
            }
        }
    }
    let mut hoisted_locals: HashSet<LocalId> = HashSet::new();
    let mut items: Vec<Hoisted> = Vec::new();
    // Read so far in the turn: a value read before it is computed keeps its
    // place.
    let mut read_so_far: HashSet<LocalId> = HashSet::new();
    visit_places(&body.blocks[shape.header], &mut |place: &Place, written: bool, _| {
        if !written {
            read_so_far.insert(place.local);
        }
    });

    let unchanged_local = |local: LocalId, hoisted: &HashSet<LocalId>| {
        hoisted.contains(&local) || (!writes.contains_key(&local) && !scoped.contains(&local))
    };
    // A place the loop cannot change: an unchanged local, its fields, and
    // memory through a reference no write in the loop can reach.
    let unchanged_place = |place: &Place, hoisted: &HashSet<LocalId>| -> bool {
        if !unchanged_local(place.local, hoisted) {
            return false;
        }
        if place.projection.iter().any(|step| !matches!(step, Projection::Field(_) | Projection::Deref)) {
            return false;
        }
        if !place.projection.contains(&Projection::Deref) {
            return true;
        }
        match locate(body, types, &facts.origins, place, 0) {
            Some(loc) => loc.root != Root::Anywhere && !written.iter().any(|(w, _)| may_alias(w, &loc, &facts)),
            None => false,
        }
    };
    let unchanged_operand = |operand: &Operand, hoisted: &HashSet<LocalId>| match operand {
        Operand::Const(_) => true,
        Operand::Copy(place) | Operand::Move(place) => unchanged_place(place, hoisted),
    };
    let leaves_nest = |operands: &[&Operand], hoisted: &HashSet<LocalId>| {
        operands.iter().all(|operand| match operand {
            Operand::Const(_) => true,
            Operand::Copy(place) | Operand::Move(place) => {
                !place.projection.is_empty() || hoisted.contains(&place.local) || !cycle_writes.contains(&place.local)
            }
        })
    };
    let movable_dest = |place: &Place, read_so_far: &HashSet<LocalId>| {
        place.projection.is_empty()
            && writes.get(&place.local) == Some(&1)
            && !types.needs_drop(body.local(place.local).ty)
            && !read_so_far.contains(&place.local)
            && !outside.contains(&place.local)
    };

    let mut block = shape.entry;
    'chain: loop {
        if block == shape.header || block == shape.step || !inside_set.contains(&block) || predecessors[block].len() != 1 {
            break;
        }
        let data = &body.blocks[block];
        for (index, stmt) in data.stmts.iter().enumerate() {
            match &stmt.kind {
                StmtKind::StorageLive(_) | StmtKind::StorageDead(_) | StmtKind::Nop => continue,
                StmtKind::Assign { place, rvalue } => {
                    let pure = match rvalue {
                        Rvalue::Use(operand) | Rvalue::Cast { operand, .. } | Rvalue::UnaryOp { operand, .. } => {
                            Some(unchanged_operand(operand, &hoisted_locals) && leaves_nest(&[operand], &hoisted_locals))
                        }
                        // A division or shift stays where the loop has it:
                        // done before the loop, it could trap when the loop
                        // would not have run.
                        Rvalue::BinaryOp { op, lhs, rhs } => Some(
                            !matches!(op, BinOp::Div | BinOp::Rem | BinOp::FloorDiv | BinOp::FloorRem | BinOp::Shl | BinOp::Shr)
                                && unchanged_operand(lhs, &hoisted_locals)
                                && unchanged_operand(rhs, &hoisted_locals)
                                && leaves_nest(&[lhs, rhs], &hoisted_locals),
                        ),
                        Rvalue::Aggregate { operands, .. } => Some(
                            operands.iter().all(|operand| unchanged_operand(operand, &hoisted_locals))
                                && leaves_nest(&operands.iter().collect::<Vec<_>>(), &hoisted_locals),
                        ),
                        Rvalue::Discriminant(read) => Some(unchanged_place(read, &hoisted_locals)),
                        // The address of a place the loop does not rebind.
                        Rvalue::Ref { place: target, .. } => Some(
                            unchanged_local(target.local, &hoisted_locals)
                                && target.projection.iter().all(|step| matches!(step, Projection::Field(_))),
                        ),
                        Rvalue::Repeat { .. } => None,
                    };
                    match pure {
                        Some(true) if movable_dest(place, &read_so_far) => {
                            items.push(Hoisted::Stmt(block, index));
                            hoisted_locals.insert(place.local);
                        }
                        Some(_) => {
                            visit_stmt(stmt, &mut |read: &Place, written: bool, _| {
                                if !written {
                                    read_so_far.insert(read.local);
                                }
                            });
                        }
                        None => break 'chain,
                    }
                }
                // An access check, a drop or a checked operation: the turn's
                // movable work ends here.
                _ => break 'chain,
            }
        }
        match &data.terminator {
            Terminator::Goto(next) => block = next.0 as usize,
            Terminator::Call {
                func:
                    FuncRef::Builtin {
                        which:
                            Builtin::SpanFrom { .. }
                            | Builtin::SpanReborrow
                            | Builtin::SpanSharedReborrow
                            | Builtin::SpanLen
                            | Builtin::ArrayLen
                            | Builtin::StringLen,
                        ..
                    },
                args,
                dest,
                next,
            } => {
                // It reads the header its argument is, or refers to.
                let reads_unchanged = args.iter().all(|arg| match arg {
                    Operand::Const(_) => true,
                    Operand::Copy(place) | Operand::Move(place) => {
                        unchanged_place(place, &hoisted_locals)
                            && (!matches!(types.kind(place_type(body, types, place)), TyKind::Ref { .. })
                                || unchanged_place(&deref_of(place), &hoisted_locals))
                    }
                });
                if reads_unchanged && movable_dest(dest, &read_so_far) {
                    items.push(Hoisted::Call(block));
                    hoisted_locals.insert(dest.local);
                } else {
                    for arg in args {
                        if let Operand::Copy(place) | Operand::Move(place) = arg {
                            read_so_far.insert(place.local);
                        }
                    }
                }
                block = next.0 as usize;
            }
            _ => break,
        }
    }
    if items.is_empty() {
        return 0;
    }

    // Before the loop, in the order the turn had them: statements gathered
    // into a block, each call ending one.
    let span = body.blocks[shape.header].terminator_span;
    let first = body.blocks.len();
    let mut blocks: Vec<BasicBlock> = Vec::new();
    let mut stmts: Vec<Stmt> = Vec::new();
    for item in &items {
        match *item {
            Hoisted::Stmt(block, index) => stmts.push(body.blocks[block].stmts[index].clone()),
            Hoisted::Call(block) => {
                let Terminator::Call { func, args, dest, .. } = body.blocks[block].terminator.clone() else {
                    unreachable!("a hoisted call")
                };
                let next = BasicBlockId((first + blocks.len() + 1) as u32);
                blocks.push(BasicBlock {
                    stmts: std::mem::take(&mut stmts),
                    terminator: Terminator::Call { func, args, dest, next },
                    terminator_span: body.blocks[block].terminator_span,
                });
            }
        }
    }
    blocks.push(BasicBlock { stmts, terminator: Terminator::Goto(BasicBlockId(shape.header as u32)), terminator_span: span });
    // Out of the loop, with the storage markers of what moved.
    for item in &items {
        match *item {
            Hoisted::Stmt(block, index) => body.blocks[block].stmts[index].kind = StmtKind::Nop,
            Hoisted::Call(block) => {
                let Terminator::Call { next, .. } = body.blocks[block].terminator else { unreachable!("a hoisted call") };
                body.blocks[block].terminator = Terminator::Goto(next);
            }
        }
    }
    for &block in &inside {
        for stmt in &mut body.blocks[block].stmts {
            if let StmtKind::StorageLive(local) | StmtKind::StorageDead(local) = stmt.kind
                && hoisted_locals.contains(&local)
            {
                stmt.kind = StmtKind::Nop;
            }
        }
    }
    let header = BasicBlockId(shape.header as u32);
    let preheader = BasicBlockId(first as u32);
    for block in 0..first {
        if !inside_set.contains(&block) {
            retarget(&mut body.blocks[block].terminator, |target| if target == header { preheader } else { target });
        }
    }
    body.blocks.extend(blocks);
    items.len()
}
