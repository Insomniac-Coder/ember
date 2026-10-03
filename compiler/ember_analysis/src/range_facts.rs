//! `[RNG-4]` — range facts over the MIR, and the checks they remove.
//!
//! For every whole-number local, and the length of every list or span a
//! function reads, the analysis keeps the interval of values it can hold at
//! each point, and facts of the form `a <= b + c` between two of them. Facts
//! come from constants, a value's type (a range type's declared range,
//! `[RNG-9]`), arithmetic on known ranges, copies and conversions, the arm of
//! a comparison a branch took, a check that passed, and a list's `len()`.
//! Loops are handled by widening at their headers and then narrowing once, so
//! a `for i in a..b` counter carries `a <= i < b` in its body. A bound that
//! moves is first widened to the nearest constant the loop holds (or one
//! either side of it), and only past the last to its type's end: a value the
//! loop only raises towards a limit it computes (`best = larger(best, x % m)`)
//! keeps that limit. A call to a function that only ever returns one of its
//! parameters unchanged gives one of those arguments (`returned_arguments`).
//!
//! What the facts show can never fail is removed: an overflow check whose
//! exact result fits its type (`[TYP-8]`), a bounds check whose index is below
//! the length, a division by a divisor that cannot be zero, a shift whose
//! amount is below the width, the `MIN / -1` check where one side excludes
//! it. A floor `//` or `%` whose check goes becomes the cheapest form that
//! means the same (`[COST-3]`): a shift or mask for a power-of-two divisor,
//! unsigned `/` or `%` when both sides are non-negative. Every removal is
//! recorded for the safety side table (`[EFF-10]`).
//!
//! The elements of a list the function makes empty itself, and changes only
//! in ways it can see (element writes, `push`, `insert`, and calls that only
//! reorder or remove), hold only values it stored: their range is the hull
//! of every stored value's, read back through `xs[i]`, a shared view's
//! `v[i]` or a shared element reference.
//!
//! A local whose address is taken mutably is not tracked, since a write
//! through the reference would be invisible. A list's length is forgotten
//! at anything that can change the list: a write to it or a place holding it,
//! a move, a mutable borrow, and — when some mutable borrow of it exists
//! anywhere in the function — every call. A list reached through a shared
//! reference cannot change while the reference lives (`[UNS-4]`).

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use ember_mir::{
    AssertKind, BinOp, Body, Builtin, CastKind, CheckKind, CheckProof, Const, FuncRef, LocalId, Operand, Place,
    Projection, RemovedCheck, Rvalue, Stmt, StmtKind, Terminator, UnOp,
};
use ember_types::{Bound, CommonTypes, Ty, TyKind, TypeTable};

use crate::regions::place_type;

/// Remove every check the range facts prove cannot fail; returns how many.
/// Returns how many checks went and how many branches were folded; a fold
/// can remove what a body reads, so its callable summary is made again.
pub fn remove_proven_checks_all(bodies: &mut [Body], types: &TypeTable, common: &CommonTypes) -> (usize, usize) {
    let returns = returned_arguments(bodies);
    bodies.iter_mut().fold((0, 0), |(checks, folds), body| {
        let (more_checks, more_folds) = remove_proven_checks(body, types, common, &returns);
        (checks + more_checks, folds + more_folds)
    })
}

// ---------------------------------------------------------------------------
// Intervals.

/// The whole numbers from `lo` to `hi`, both included.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct Interval {
    pub lo: i128,
    pub hi: i128,
}

impl Interval {
    fn exact(value: i128) -> Interval {
        Interval { lo: value, hi: value }
    }

    fn meet(self, other: Interval) -> Option<Interval> {
        let meet = Interval { lo: self.lo.max(other.lo), hi: self.hi.min(other.hi) };
        (meet.lo <= meet.hi).then_some(meet)
    }

    fn hull(self, other: Interval) -> Interval {
        Interval { lo: self.lo.min(other.lo), hi: self.hi.max(other.hi) }
    }

    fn within(self, other: Interval) -> bool {
        other.lo <= self.lo && self.hi <= other.hi
    }

    fn contains(self, value: i128) -> bool {
        self.lo <= value && value <= self.hi
    }
}

/// The values a type holds: an integer's whole range, a `char`'s scalar
/// interval (`[TYP-3]`), or a range type's declared one (`[RNG-9]`). `None`
/// for anything else, and for `u128`, whose top half no `i128` holds.
pub(crate) fn type_range(types: &TypeTable, ty: Ty) -> Option<Interval> {
    match types.kind(ty) {
        TyKind::Int(_) | TyKind::Uint(_) => {
            let max = i128::try_from(ember_types::int_max(types, ty)?).ok()?;
            let lo = if ember_types::is_signed(types, ty) == Some(true) { -max - 1 } else { 0 };
            Some(Interval { lo, hi: max })
        }
        // `[TYP-3]` requires every `char` to remain a Unicode scalar through
        // unsafe and foreign boundaries. `[FFI-8]` maps `char32_t` to `u32`,
        // whose full integer range stays above. The interval deliberately
        // includes the surrogate gap: it is a conservative bound, not a
        // scalar-validity check.
        TyKind::Char => Some(Interval { lo: 0, hi: 0x10FFFF }),
        TyKind::Range(id) => {
            let def = types.range_def(*id);
            let repr = type_range(types, def.repr)?;
            let (Bound::Int(lo), Bound::Int(hi)) = (def.lo, def.hi) else { return None };
            let hi = if def.inclusive { hi } else { hi.checked_sub(1)? };
            repr.meet(Interval { lo, hi })
        }
        _ => None,
    }
}

/// The fewest bytes a value of `ty` takes: its scalars' widths, summed
/// through structs, tuples and fixed arrays; none for what has no known
/// width, so the answer is never more than the C type's size.
fn min_size(types: &TypeTable, ty: Ty) -> u64 {
    match types.kind(ty) {
        TyKind::Bool => 1,
        TyKind::Char => 4,
        TyKind::Float(ember_types::FloatTy::F16) => 2,
        TyKind::Float(ember_types::FloatTy::F32) => 4,
        TyKind::Float(ember_types::FloatTy::F64) => 8,
        TyKind::Struct(id) => types.struct_def(*id).fields.iter().map(|field| min_size(types, field.ty)).fold(0, u64::saturating_add),
        TyKind::Tuple(items) => items.iter().map(|&item| min_size(types, item)).fold(0, u64::saturating_add),
        TyKind::Array { elem, len } => min_size(types, *elem).saturating_mul(*len),
        _ => ember_types::bit_width(types, representation(types, ty)).map_or(0, |bits| bits / 8),
    }
}

/// The integer type a value of `ty` is represented by.
fn representation(types: &TypeTable, ty: Ty) -> Ty {
    match types.kind(ty) {
        TyKind::Range(id) => types.range_def(*id).repr,
        _ => ty,
    }
}

/// A constant's bits as a number of type `ty`.
pub(crate) fn constant(types: &TypeTable, value: u128, ty: Ty) -> Option<i128> {
    let ty = representation(types, ty);
    let width = ember_types::bit_width(types, ty)?;
    let signed = ember_types::is_signed(types, ty)?;
    if width == 128 {
        return if signed { Some(value as i128) } else { i128::try_from(value).ok() };
    }
    let low = value & ((1u128 << width) - 1);
    Some(if signed && low >> (width - 1) == 1 { low as i128 - (1i128 << width) } else { low as i128 })
}

fn corners(a: Interval, b: Interval, f: impl Fn(i128, i128) -> Option<i128>) -> Option<Interval> {
    let values = [f(a.lo, b.lo)?, f(a.lo, b.hi)?, f(a.hi, b.lo)?, f(a.hi, b.hi)?];
    Some(Interval { lo: *values.iter().min()?, hi: *values.iter().max()? })
}

fn floor_div(x: i128, y: i128) -> Option<i128> {
    let q = x.checked_div(y)?;
    if x % y != 0 && ((x < 0) != (y < 0)) { q.checked_sub(1) } else { Some(q) }
}

/// The exact results of `a op b` for every value of each operand; `None`
/// where nothing useful is known. `width` is the operands' width, for shifts.
fn arithmetic(op: BinOp, a: Interval, b: Interval, width: u32) -> Option<Interval> {
    // A divisor of one sign: quotients are monotone in each operand.
    let one_signed = b.lo > 0 || b.hi < 0;
    // `|a % b| < |b|` whatever the signs.
    let magnitude = || b.lo.checked_abs()?.max(b.hi.checked_abs()?).checked_sub(1).filter(|m| *m >= 0);
    match op {
        BinOp::Add => corners(a, b, i128::checked_add),
        BinOp::Sub => corners(a, b, i128::checked_sub),
        BinOp::Mul => corners(a, b, i128::checked_mul),
        BinOp::Div if one_signed => corners(a, b, i128::checked_div),
        BinOp::FloorDiv if one_signed => corners(a, b, floor_div),
        // C's remainder has the dividend's sign.
        BinOp::Rem => {
            let m = magnitude()?;
            Some(if a.lo >= 0 {
                Interval { lo: 0, hi: a.hi.min(m) }
            } else if a.hi <= 0 {
                Interval { lo: a.lo.max(-m), hi: 0 }
            } else {
                Interval { lo: -m, hi: m }
            })
        }
        // `[TYP-28]` — a floor remainder has the divisor's sign.
        BinOp::FloorRem => {
            if b.lo > 0 {
                let hi = if a.lo >= 0 { a.hi.min(b.hi - 1) } else { b.hi - 1 };
                Some(Interval { lo: 0, hi })
            } else if b.hi < 0 {
                let lo = if a.hi <= 0 { a.lo.max(b.lo + 1) } else { b.lo + 1 };
                Some(Interval { lo, hi: 0 })
            } else {
                let m = magnitude()?;
                Some(Interval { lo: -m, hi: m })
            }
        }
        BinOp::BitAnd => {
            if a.lo == a.hi && b.lo == b.hi {
                return Some(Interval::exact(a.lo & b.lo));
            }
            // Every bit of the result is a bit of a non-negative operand.
            match (a.lo >= 0, b.lo >= 0) {
                (true, true) => Some(Interval { lo: 0, hi: a.hi.min(b.hi) }),
                (true, false) => Some(Interval { lo: 0, hi: a.hi }),
                (false, true) => Some(Interval { lo: 0, hi: b.hi }),
                (false, false) => None,
            }
        }
        BinOp::BitOr | BinOp::BitXor => {
            if a.lo == a.hi && b.lo == b.hi {
                let value = if op == BinOp::BitOr { a.lo | b.lo } else { a.lo ^ b.lo };
                return Some(Interval::exact(value));
            }
            if a.lo < 0 || b.lo < 0 {
                return None;
            }
            let top = a.hi.max(b.hi);
            let bits = 128 - top.leading_zeros();
            Some(Interval { lo: 0, hi: 1i128.checked_shl(bits)?.checked_sub(1)? })
        }
        BinOp::Shl | BinOp::Shr => {
            if b.lo < 0 || b.hi >= i128::from(width) {
                return None;
            }
            let pow = |k: i128| 1i128.checked_shl(u32::try_from(k).ok()?).filter(|p| *p > 0);
            if op == BinOp::Shl {
                // The exact product, which is what the overflow check compares.
                corners(a, b, |x, k| x.checked_mul(pow(k)?))
            } else {
                corners(a, b, |x, k| Some(x >> u32::try_from(k).ok()?))
            }
        }
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Facts.

/// A value the facts name: a tracked local, or a view's length.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub(crate) enum Var {
    Local(LocalId),
    Len(usize),
}

/// One side of a comparison.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Term {
    Var(Var),
    Const(i128),
}

/// What a `bool` local is known to hold.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum BoolFact {
    Known(bool),
    /// The comparison it holds, while neither side has changed.
    Cmp { op: BinOp, lhs: Term, rhs: Term },
}

#[derive(Clone, PartialEq, Eq, Default, Debug)]
pub(crate) struct State {
    /// Narrower than the value's type; absent means the type's range.
    ranges: BTreeMap<Var, Interval>,
    /// `a <= b + c`, the tightest `c` known.
    rels: BTreeMap<(Var, Var), i128>,
    bools: BTreeMap<LocalId, BoolFact>,
}

impl State {
    fn join(&self, other: &State) -> State {
        let ranges = self
            .ranges
            .iter()
            .filter_map(|(var, a)| other.ranges.get(var).map(|b| (*var, a.hull(*b))))
            .collect();
        let rels = self
            .rels
            .iter()
            .filter_map(|(key, a)| other.rels.get(key).map(|b| (*key, (*a).max(*b))))
            .collect();
        let bools = self
            .bools
            .iter()
            .filter(|(local, fact)| other.bools.get(local) == Some(fact))
            .map(|(local, fact)| (*local, *fact))
            .collect();
        State { ranges, rels, bools }
    }

    /// `join`, with any bound that moved pushed to the type's end, and any
    /// relation that loosened dropped: a loop header reaches a fixpoint. Only
    /// a local the loop itself writes is widened (`changed`); any other grew
    /// on the way in, from an enclosing loop that widens it, and is joined, so
    /// an outer counter keeps its range inside an inner loop.
    fn widen(&self, next: &State, analysis: &Analysis, changed: &HashSet<LocalId>, thresholds: &[i128]) -> State {
        let mut ranges = BTreeMap::new();
        for (var, old) in &self.ranges {
            let Some(new) = next.ranges.get(var) else { continue };
            let full = analysis.var_range(*var);
            let widened = match var {
                Var::Local(local) => changed.contains(local),
                Var::Len(_) => true,
            };
            // The nearest threshold past the moved bound, else the type's end.
            let up = || thresholds.iter().copied().find(|&t| t >= new.hi && t < full.hi).unwrap_or(full.hi);
            let down = || thresholds.iter().rev().copied().find(|&t| t <= new.lo && t > full.lo).unwrap_or(full.lo);
            let lo = if new.lo < old.lo { if widened { down() } else { new.lo } } else { old.lo };
            let hi = if new.hi > old.hi { if widened { up() } else { new.hi } } else { old.hi };
            if (Interval { lo, hi }) != full {
                ranges.insert(*var, Interval { lo, hi });
            }
        }
        let rels = self
            .rels
            .iter()
            .filter(|(key, c)| next.rels.get(key).is_some_and(|n| n <= c))
            .map(|(key, c)| (*key, *c))
            .collect();
        let bools = self
            .bools
            .iter()
            .filter(|(local, fact)| next.bools.get(local) == Some(fact))
            .map(|(local, fact)| (*local, *fact))
            .collect();
        State { ranges, rels, bools }
    }

    fn forget(&mut self, var: Var) {
        // What held through `var` still holds between the others once it is
        // gone: `a <= var + c1` and `var <= b + c2` give `a <= b + c1 + c2`.
        // A temporary's storage ends right after it is read, and the chain
        // from a loop's bound to a list's length would break with it.
        let into: Vec<(Var, i128)> =
            self.rels.iter().filter(|((a, b), _)| *b == var && *a != var).map(|((a, _), c)| (*a, *c)).collect();
        let out: Vec<(Var, i128)> =
            self.rels.iter().filter(|((a, b), _)| *a == var && *b != var).map(|((_, b), c)| (*b, *c)).collect();
        for &(a, c1) in &into {
            for &(b, c2) in &out {
                if a == b {
                    continue;
                }
                if let Some(c) = c1.checked_add(c2) {
                    let entry = self.rels.entry((a, b)).or_insert(c);
                    *entry = (*entry).min(c);
                }
            }
        }
        self.ranges.remove(&var);
        self.rels.retain(|(a, b), _| *a != var && *b != var);
        self.bools.retain(|_, fact| match fact {
            BoolFact::Cmp { lhs, rhs, .. } => *lhs != Term::Var(var) && *rhs != Term::Var(var),
            BoolFact::Known(_) => true,
        });
    }
}

/// A list or span whose length the facts can name: its place, reached from a
/// local through struct fields and references only.
struct View {
    place: Place,
    /// Some mutable borrow of it, or of the reference reaching it, exists in
    /// the function: any call may change it.
    lent: bool,
}

/// Where values go into the elements of a list whose elements are followed.
#[derive(Clone, Copy, Debug)]
enum Store {
    /// Statement `.1` of block `.0` writes one element.
    Element(usize, usize),
    /// The call ending block `.0` adds its argument `.1` (`push`, `insert`).
    Added(usize, usize),
    /// A call adds elements from elsewhere (`extend`): any value.
    Unknown,
}

/// The integer lists whose elements the facts follow, and what reads them.
#[derive(Default)]
struct Lists {
    /// Per list: every store into its elements.
    stores: BTreeMap<LocalId, Vec<Store>>,
    /// A shared view of a list (`as_span`): the list.
    views: BTreeMap<LocalId, LocalId>,
    /// A shared reference to one element of a list or of such a view: the
    /// lists its writes reach (more than one when `a.chain(b)`'s loops share
    /// it, ADR-083).
    items: BTreeMap<LocalId, BTreeSet<LocalId>>,
}

pub(crate) struct Analysis<'a> {
    body: &'a Body,
    types: &'a TypeTable,
    /// Per local: a whole integer local no reference can write.
    tracked: Vec<bool>,
    views: Vec<View>,
    lists: Lists,
    /// Per followed list: the range of its elements, narrower than the type's.
    elements: BTreeMap<LocalId, Interval>,
    /// Each `count = RangeCount(start, stop, step)` whose four locals are
    /// written once: `(count, start, stop, step)`.
    counts: Vec<(LocalId, LocalId, LocalId, LocalId)>,
    /// Each view's length range: its elements' bytes are at most
    /// `PTRDIFF_MAX`, as no list holds more (`[HEAP-8]`'s limit,
    /// `vec_max_elems`) and no C object is larger; a view is of a list, a
    /// fixed array or a slice of one. Elements of no size count as one byte,
    /// as the runtime counts them.
    len_ranges: Vec<Interval>,
    /// The state on entry to each block; `None` when unreachable.
    entry: Vec<Option<State>>,
    /// Ranges known at a loop header whatever the path in: each counted
    /// loop's running totals (`accumulator_bounds`).
    seeds: Vec<(usize, Var, Interval)>,
    /// Per function symbol, the parameters (by position) it may return: it
    /// returns nothing else, and changes none of them.
    returns: &'a HashMap<String, Vec<usize>>,
}

/// Per function symbol, the parameters (by position) every value it returns
/// is one of, unchanged: no parameter is written or lent mutably, and the
/// return place is only ever a copy of one and never lent mutably. Bodies sharing a symbol must
/// agree, or the symbol has no entry.
pub(crate) fn returned_arguments(bodies: &[Body]) -> HashMap<String, Vec<usize>> {
    let mut found: HashMap<String, Option<Vec<usize>>> = HashMap::new();
    for body in bodies {
        let summary = arguments_returned(body);
        found
            .entry(body.symbol.clone())
            .and_modify(|seen| {
                if *seen != summary {
                    *seen = None;
                }
            })
            .or_insert(summary);
    }
    found.into_iter().filter_map(|(symbol, summary)| Some((symbol, summary?))).collect()
}

fn arguments_returned(body: &Body) -> Option<Vec<usize>> {
    let ret = LocalId(0);
    let is_param = |local: LocalId| (1..=body.arg_count).contains(&(local.0 as usize));
    let mut returned = BTreeSet::new();
    for data in &body.blocks {
        for stmt in &data.stmts {
            match &stmt.kind {
                StmtKind::Assign { place, rvalue } => {
                    if place.local == ret {
                        let Rvalue::Use(Operand::Copy(source) | Operand::Move(source)) = rvalue else { return None };
                        if !place.projection.is_empty() || !source.projection.is_empty() || !is_param(source.local) {
                            return None;
                        }
                        returned.insert(source.local.0 as usize - 1);
                    } else if is_param(place.local) {
                        return None;
                    }
                    if let Rvalue::Ref { place: target, mutable: true } = rvalue
                        && (target.local == ret || is_param(target.local))
                    {
                        return None;
                    }
                }
                StmtKind::CheckedBinaryOp { dest, overflow, .. } => {
                    if [dest.local, overflow.local].iter().any(|&local| local == ret || is_param(local)) {
                        return None;
                    }
                }
                _ => {}
            }
        }
        if let Terminator::Call { dest, .. } = &data.terminator
            && (dest.local == ret || is_param(dest.local))
        {
            return None;
        }
    }
    (!returned.is_empty()).then(|| returned.into_iter().collect())
}

/// Rounds before a body is left without facts: a guard, since widening
/// bounds every chain.
const MAX_ROUNDS: usize = 200;

/// The most thresholds one loop's widening tries.
const MAX_THRESHOLDS: usize = 64;

impl<'a> Analysis<'a> {
    /// The facts of `body`, or `None` when they did not settle.
    pub(crate) fn run(
        body: &'a Body,
        types: &'a TypeTable,
        common: &CommonTypes,
        returns: &'a HashMap<String, Vec<usize>>,
    ) -> Option<Analysis<'a>> {
        let isize_max = type_range(types, common.isize)?.hi;
        let mut analysis = Analysis {
            body,
            types,
            tracked: Vec::new(),
            views: Vec::new(),
            lists: Lists::default(),
            elements: BTreeMap::new(),
            counts: Vec::new(),
            len_ranges: Vec::new(),
            entry: Vec::new(),
            seeds: Vec::new(),
            returns,
        };
        analysis.tracked = analysis.tracked_locals();
        analysis.views = analysis.collect_views();
        analysis.len_ranges = analysis
            .views
            .iter()
            .map(|view| {
                let size = match types.kind(place_type(body, types, &view.place)) {
                    TyKind::Vec { elem, .. } | TyKind::Span { elem, .. } => min_size(types, *elem).max(1),
                    _ => 1,
                };
                Interval { lo: 0, hi: isize_max / i128::from(size) }
            })
            .collect();
        analysis.lists = analysis.element_lists();
        analysis.counts = analysis.range_counts();
        analysis.solve()?;
        // Element ranges from what the stores wrote. Each round's come from
        // facts that assumed the last round's, so each is sound, and they
        // only narrow; a second round reaches a store that reads its own list.
        for _ in 0..2 {
            let elements = analysis.element_ranges();
            if elements == analysis.elements {
                break;
            }
            analysis.elements = elements;
            analysis.solve()?;
        }
        // Widening loses a running total's range; the loop's trip count
        // gives it back, and a second solve uses it.
        let seeds = analysis.accumulator_bounds();
        if !seeds.is_empty() {
            analysis.seeds = seeds;
            analysis.solve()?;
        }
        Some(analysis)
    }

    fn tracked_locals(&self) -> Vec<bool> {
        let mut tracked: Vec<bool> =
            self.body.locals.iter().map(|decl| type_range(self.types, decl.ty).is_some()).collect();
        for block in &self.body.blocks {
            for stmt in &block.stmts {
                if let StmtKind::Assign { rvalue: Rvalue::Ref { place, .. }, .. } = &stmt.kind {
                    tracked[place.local.0 as usize] = false;
                }
            }
        }
        tracked
    }

    fn collect_views(&self) -> Vec<View> {
        // Every list whose length field is read, or whose `len()` is taken.
        let mut places: Vec<Place> = Vec::new();
        let len_field = |place: &Place| match place.projection.split_last() {
            Some((Projection::Field(1), view)) => Some(Place { local: place.local, projection: view.to_vec() }),
            _ => None,
        };
        for block in &self.body.blocks {
            for stmt in &block.stmts {
                for_each_place(stmt, &mut |place: &Place| places.extend(len_field(place)));
            }
            match &block.terminator {
                Terminator::Assert { msg: AssertKind::Bounds { len: Operand::Copy(place), .. }, .. } => {
                    places.extend(len_field(place));
                }
                Terminator::Call { func: FuncRef::Builtin { which, .. }, args, .. } if is_len(which) => {
                    if let Some(Operand::Copy(place) | Operand::Move(place)) = args.first() {
                        places.push(place.clone());
                    }
                }
                _ => {}
            }
        }
        places.sort();
        places.dedup();
        places
            .into_iter()
            .filter_map(|place| {
                let frozen = self.view_path(&place)?;
                // Reached through a shared reference, it cannot change while that lives.
                Some(View { lent: !frozen && self.lent(&place), place })
            })
            .collect()
    }

    /// The integer lists whose elements the facts can follow. A list local
    /// (not a parameter) qualifies when every value it holds is one this
    /// function stored: it is only ever made by `Array()`, written by element
    /// (`xs[i] = v`), and lent mutably only to `push`, `insert`, `extend` and
    /// the calls that only reorder or remove (`sort`, `pop`, ...), each
    /// through a reference written once and used for nothing else. A copy of
    /// its header, any other mutable borrow, or any other way in, and it is
    /// not followed. Its shared views and element references read it.
    fn element_lists(&self) -> Lists {
        let body = self.body;
        let mut stores: BTreeMap<LocalId, Vec<Store>> = BTreeMap::new();
        for (index, decl) in body.locals.iter().enumerate().skip(body.arg_count + 1) {
            if let TyKind::Vec { elem, text: false } = *self.types.kind(decl.ty) {
                if type_range(self.types, elem).is_some() {
                    stores.insert(LocalId(index as u32), Vec::new());
                }
            }
        }
        if stores.is_empty() {
            return Lists::default();
        }
        // How many times each local is written whole.
        let mut writes = vec![0usize; body.locals.len()];
        let mut count = |place: &Place| {
            if place.projection.is_empty() {
                writes[place.local.0 as usize] += 1;
            }
        };
        for block in &body.blocks {
            for stmt in &block.stmts {
                match &stmt.kind {
                    StmtKind::Assign { place, .. } => count(place),
                    StmtKind::CheckedBinaryOp { dest, overflow, .. } => {
                        count(dest);
                        count(overflow);
                    }
                    _ => {}
                }
            }
            if let Terminator::Call { dest, .. } = &block.terminator {
                count(dest);
            }
        }
        let once = |local: LocalId| writes[local.0 as usize] == 1;
        let element = |place: &Place| matches!(place.projection.as_slice(), [Projection::Index(_) | Projection::ConstIndex(_)]);
        let mut bad: BTreeSet<LocalId> = BTreeSet::new();
        // `t = &mut xs` (a lender) and `t = &xs`, each `t` written once.
        let mut lenders: BTreeMap<LocalId, LocalId> = BTreeMap::new();
        let mut shared: BTreeMap<LocalId, LocalId> = BTreeMap::new();
        for block in &body.blocks {
            for stmt in &block.stmts {
                let StmtKind::Assign { place, rvalue: Rvalue::Ref { place: target, mutable } } = &stmt.kind else { continue };
                if !stores.contains_key(&target.local) {
                    continue;
                }
                let whole = target.projection.is_empty() && place.projection.is_empty() && once(place.local);
                match (*mutable, whole) {
                    (true, true) => {
                        lenders.insert(place.local, target.local);
                    }
                    (true, false) => {
                        bad.insert(target.local);
                    }
                    (false, true) => {
                        shared.insert(place.local, target.local);
                    }
                    (false, false) => {}
                }
            }
        }
        // Shared views `v = as_span(t)`, then element references `r = &xs[i]`
        // or `r = &v[i]`.
        let mut views: BTreeMap<LocalId, LocalId> = BTreeMap::new();
        for block in &body.blocks {
            if let Terminator::Call { func: FuncRef::Builtin { which: Builtin::SpanFrom { mutable: false }, .. }, args, dest, .. } =
                &block.terminator
                && let [Operand::Copy(source) | Operand::Move(source)] = args.as_slice()
                && let Some(&list) = shared.get(&source.local)
                && source.projection.is_empty()
                && dest.projection.is_empty()
                && once(dest.local)
            {
                views.insert(dest.local, list);
            }
        }
        // A local every whole write of which is `&xs[i]`, `&v[i]` or a copy of
        // such a local: the lists those reach. A write of anything else, or
        // a partial one, leaves it out.
        let mut sources: BTreeMap<LocalId, Vec<Result<LocalId, LocalId>>> = BTreeMap::new();
        let mut excluded: BTreeSet<LocalId> = BTreeSet::new();
        for block in &body.blocks {
            for stmt in &block.stmts {
                match &stmt.kind {
                    StmtKind::Assign { place, rvalue } if place.projection.is_empty() => {
                        let source = match rvalue {
                            Rvalue::Ref { place: target, mutable: false } if element(target) => {
                                if stores.contains_key(&target.local) {
                                    Some(Ok(target.local))
                                } else {
                                    views.get(&target.local).map(|&list| Ok(list))
                                }
                            }
                            Rvalue::Use(Operand::Copy(from) | Operand::Move(from))
                                if from.projection.is_empty()
                                    && matches!(self.types.kind(body.local(from.local).ty), TyKind::Ref { mutable: false, .. }) =>
                            {
                                Some(Err(from.local))
                            }
                            _ => None,
                        };
                        match source {
                            Some(source) => sources.entry(place.local).or_default().push(source),
                            None => {
                                excluded.insert(place.local);
                            }
                        }
                    }
                    StmtKind::Assign { place, .. } => {
                        excluded.insert(place.local);
                    }
                    StmtKind::CheckedBinaryOp { dest, overflow, .. } => {
                        excluded.insert(dest.local);
                        excluded.insert(overflow.local);
                    }
                    _ => {}
                }
            }
            if let Terminator::Call { dest, .. } = &block.terminator {
                excluded.insert(dest.local);
            }
        }
        sources.retain(|local, _| !excluded.contains(local));
        let mut items: BTreeMap<LocalId, BTreeSet<LocalId>> = BTreeMap::new();
        // A copy's lists are its source's: settle them until nothing changes;
        // a copy of a local that is not an item leaves the copy out.
        loop {
            let mut changed = false;
            for (&local, writes) in &sources {
                let mut lists = BTreeSet::new();
                let mut known = true;
                for source in writes {
                    match source {
                        Ok(list) => {
                            lists.insert(*list);
                        }
                        Err(from) => match items.get(from) {
                            Some(from_lists) => lists.extend(from_lists.iter().copied()),
                            None => known = false,
                        },
                    }
                }
                if known && items.get(&local) != Some(&lists) {
                    items.insert(local, lists);
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }
        // Copies of copies that never settled (a copy of a non-item) stay out.
        items.retain(|local, _| {
            sources[local].iter().all(|source| match source {
                Ok(_) => true,
                Err(from) => sources.contains_key(from),
            })
        });
        // Every way in.
        for (b, block) in body.blocks.iter().enumerate() {
            for (s, stmt) in block.stmts.iter().enumerate() {
                let defines_lender = matches!(&stmt.kind,
                    StmtKind::Assign { place, rvalue: Rvalue::Ref { mutable: true, .. } } if lenders.contains_key(&place.local));
                if !defines_lender {
                    for_each_place(stmt, &mut |place: &Place| {
                        if let Some(&list) = lenders.get(&place.local) {
                            bad.insert(list);
                        }
                    });
                }
                match &stmt.kind {
                    StmtKind::Assign { place, rvalue } => {
                        if let Some(list) = stores.get_mut(&place.local) {
                            if element(place) {
                                list.push(Store::Element(b, s));
                            } else {
                                bad.insert(place.local);
                            }
                        }
                        for operand in rvalue_operands(rvalue) {
                            if let Operand::Copy(read) = operand
                                && read.projection.is_empty()
                                && stores.contains_key(&read.local)
                            {
                                bad.insert(read.local);
                            }
                        }
                    }
                    StmtKind::CheckedBinaryOp { dest, overflow, .. } => {
                        if let Some(list) = stores.get_mut(&dest.local) {
                            if element(dest) {
                                list.push(Store::Element(b, s));
                            } else {
                                bad.insert(dest.local);
                            }
                        }
                        if stores.contains_key(&overflow.local) {
                            bad.insert(overflow.local);
                        }
                    }
                    _ => {}
                }
            }
            match &block.terminator {
                Terminator::Call { func, args, dest, .. } => {
                    let which = match func {
                        FuncRef::Builtin { which, .. } => Some(which),
                        _ => None,
                    };
                    if stores.contains_key(&dest.local)
                        && !(dest.projection.is_empty() && matches!(which, Some(Builtin::ArrayNew)))
                    {
                        bad.insert(dest.local);
                    }
                    if let Some(&list) = lenders.get(&dest.local) {
                        bad.insert(list);
                    }
                    for (k, arg) in args.iter().enumerate() {
                        let (Operand::Copy(place) | Operand::Move(place)) = arg else { continue };
                        if let Some(&list) = lenders.get(&place.local) {
                            let store = match which {
                                _ if k != 0 || !place.projection.is_empty() => Err(()),
                                Some(Builtin::ArrayPush | Builtin::ArrayInsert) => Ok(Some(Store::Added(b, 1))),
                                Some(Builtin::ArrayExtend) => Ok(Some(Store::Unknown)),
                                Some(which) if reorders_or_removes(which) => Ok(None),
                                _ => Err(()),
                            };
                            match store {
                                Ok(Some(store)) => stores.get_mut(&list).expect("a lender's list").push(store),
                                Ok(None) => {}
                                Err(()) => {
                                    bad.insert(list);
                                }
                            }
                        } else if matches!(arg, Operand::Copy(_))
                            && place.projection.is_empty()
                            && stores.contains_key(&place.local)
                            && !matches!(which, Some(Builtin::ArrayLen | Builtin::ArrayCapacity))
                        {
                            bad.insert(place.local);
                        }
                    }
                }
                Terminator::SwitchInt { discr: operand, .. } | Terminator::Assert { cond: operand, .. } => {
                    if let Operand::Copy(place) | Operand::Move(place) = operand
                        && let Some(&list) = lenders.get(&place.local)
                    {
                        bad.insert(list);
                    }
                }
                _ => {}
            }
        }
        stores.retain(|list, _| !bad.contains(list));
        views.retain(|_, list| stores.contains_key(list));
        items.retain(|_, lists| lists.iter().all(|list| stores.contains_key(list)));
        Lists { stores, views, items }
    }

    /// Per followed list, the hull of every value its stores write under the
    /// current facts, met with the last round's.
    fn element_ranges(&self) -> BTreeMap<LocalId, Interval> {
        let mut out = BTreeMap::new();
        for (&list, stores) in &self.lists.stores {
            let TyKind::Vec { elem, .. } = *self.types.kind(self.body.local(list).ty) else { continue };
            let Some(full) = type_range(self.types, elem) else { continue };
            let mut range: Option<Interval> = None;
            for store in stores {
                // A block the facts show no run reaches stores nothing.
                let value = match *store {
                    Store::Element(block, index) => {
                        let Some(state) = self.state_before(block, index) else { continue };
                        match &self.body.blocks[block].stmts[index].kind {
                            StmtKind::Assign { rvalue, .. } => self.value(&state, rvalue, elem).0,
                            // Stored only if its check passes: a failing one
                            // ends the program before anything reads it.
                            StmtKind::CheckedBinaryOp { op, lhs, rhs, .. } => {
                                self.arithmetic_value(&state, *op, lhs, rhs, full, true).0
                            }
                            _ => None,
                        }
                    }
                    Store::Added(block, arg) => {
                        let data = &self.body.blocks[block];
                        let Some(state) = self.state_before(block, data.stmts.len()) else { continue };
                        let Terminator::Call { args, .. } = &data.terminator else { continue };
                        self.operand_range(&state, &args[arg])
                    }
                    Store::Unknown => None,
                };
                let value = value.and_then(|value| value.meet(full)).unwrap_or(full);
                range = Some(range.map_or(value, |range| range.hull(value)));
            }
            // With no store the list is always empty: nothing is read from it.
            let Some(mut range) = range else { continue };
            if let Some(old) = self.elements.get(&list) {
                range = range.meet(*old).unwrap_or(range);
            }
            if range != full {
                out.insert(list, range);
            }
        }
        out
    }

    /// Each `count = RangeCount(start, stop, step)` whose count, start, stop
    /// and step are whole locals written only there and once each.
    fn range_counts(&self) -> Vec<(LocalId, LocalId, LocalId, LocalId)> {
        let mut writes = vec![0usize; self.body.locals.len()];
        for block in &self.body.blocks {
            for stmt in &block.stmts {
                match &stmt.kind {
                    StmtKind::Assign { place, .. } => writes[place.local.0 as usize] += 1,
                    StmtKind::CheckedBinaryOp { dest, overflow, .. } => {
                        writes[dest.local.0 as usize] += 1;
                        writes[overflow.local.0 as usize] += 1;
                    }
                    _ => {}
                }
            }
            if let Terminator::Call { dest, .. } = &block.terminator {
                writes[dest.local.0 as usize] += 1;
            }
        }
        let whole = |operand: &Operand| match operand {
            Operand::Copy(place) | Operand::Move(place) if place.projection.is_empty() => Some(place.local),
            _ => None,
        };
        let mut counts = Vec::new();
        for block in &self.body.blocks {
            let Terminator::Call { func: FuncRef::Builtin { which: Builtin::RangeCount, .. }, args, dest, .. } = &block.terminator
            else {
                continue;
            };
            let [start, stop, step] = args.as_slice() else { continue };
            let (Some(start), Some(stop), Some(step)) = (whole(start), whole(stop), whole(step)) else { continue };
            let locals = [dest.local, start, stop, step];
            if dest.projection.is_empty()
                && locals.iter().all(|&local| writes[local.0 as usize] == 1 && self.tracked[local.0 as usize])
            {
                counts.push((dest.local, start, stop, step));
            }
        }
        counts
    }

    /// `RangeNth(start, step, index)`'s range and relations, when a count of
    /// the same `start` and `step` towards `stop` bounds `index` and `step`
    /// is positive: `start <= value <= stop - 1`.
    fn range_nth_value(&self, state: &State, args: &[Operand]) -> Option<(Interval, Vec<(Var, i128, i128)>)> {
        let whole = |operand: &Operand| match operand {
            Operand::Copy(place) | Operand::Move(place) if place.projection.is_empty() => Some(place.local),
            _ => None,
        };
        let [start, step, index] = args else { return None };
        let (start, step, index) = (whole(start)?, whole(step)?, whole(index)?);
        let &(count, _, stop, _) = self.counts.iter().find(|&&(_, s, _, k)| s == start && k == step)?;
        if self.range(state, Var::Local(step)).lo < 1
            || self.decide(state, BinOp::Lt, Term::Var(Var::Local(index)), Term::Var(Var::Local(count))) != Some(true)
        {
            return None;
        }
        let low = self.range(state, Var::Local(start)).lo;
        let high = self.range(state, Var::Local(stop)).hi.checked_sub(1)?;
        let range = Interval { lo: low, hi: high.max(low) };
        Some((range, vec![(Var::Local(stop), -1, i128::MAX), (Var::Local(start), i128::MAX, 0)]))
    }

    /// Whether `place` is a list or span reached from a local through struct
    /// fields and references only; `Some(true)` when through a shared one.
    fn view_path(&self, place: &Place) -> Option<bool> {
        if !matches!(self.types.kind(place_type(self.body, self.types, place)), TyKind::Vec { .. } | TyKind::Span { .. }) {
            return None;
        }
        let mut frozen = false;
        for k in 0..place.projection.len() {
            let prefix = Place { local: place.local, projection: place.projection[..k].to_vec() };
            match (&place.projection[k], self.types.kind(place_type(self.body, self.types, &prefix))) {
                (Projection::Deref, TyKind::Ref { mutable, .. }) => frozen |= !*mutable,
                (Projection::Field(_), TyKind::Struct(_) | TyKind::Tuple(_)) => {}
                _ => return None,
            }
        }
        Some(frozen)
    }

    /// Whether a mutable borrow of `view`, or of anything holding it, exists,
    /// or the reference it is reached through is passed on.
    fn lent(&self, view: &Place) -> bool {
        let through_reference = view.projection.contains(&Projection::Deref);
        self.body.blocks.iter().any(|block| {
            let stmt_lends = block.stmts.iter().any(|stmt| match &stmt.kind {
                StmtKind::Assign { rvalue: Rvalue::Ref { place, mutable: true }, .. } => overlaps(place, view),
                StmtKind::Assign { rvalue, .. } if through_reference => rvalue_operands(rvalue)
                    .any(|operand| matches!(operand, Operand::Copy(p) | Operand::Move(p) if p.local == view.local && p.projection.is_empty())),
                _ => false,
            });
            let call_lends = through_reference
                && matches!(&block.terminator, Terminator::Call { args, .. }
                    if args.iter().any(|operand| matches!(operand, Operand::Copy(p) | Operand::Move(p) if p.local == view.local && p.projection.is_empty())));
            stmt_lends || call_lends
        })
    }

    fn view_of(&self, place: &Place) -> Option<usize> {
        self.views.iter().position(|view| view.place == *place)
    }

    /// The view whose length `place` is (`view.1`).
    fn len_of(&self, place: &Place) -> Option<usize> {
        let (Projection::Field(1), view) = place.projection.split_last()? else { return None };
        self.view_of(&Place { local: place.local, projection: view.to_vec() })
    }

    fn var_range(&self, var: Var) -> Interval {
        match var {
            Var::Local(local) => type_range(self.types, self.body.local(local).ty).expect("a tracked local is an integer"),
            Var::Len(index) => self.len_ranges[index],
        }
    }

    fn range(&self, state: &State, var: Var) -> Interval {
        state.ranges.get(&var).copied().unwrap_or_else(|| self.var_range(var))
    }

    fn term(&self, operand: &Operand) -> Option<Term> {
        match operand {
            Operand::Const(Const::Int { value, ty }) => Some(Term::Const(constant(self.types, *value, *ty)?)),
            Operand::Copy(place) | Operand::Move(place) => {
                if place.projection.is_empty() && self.tracked[place.local.0 as usize] {
                    Some(Term::Var(Var::Local(place.local)))
                } else {
                    self.len_of(place).map(|view| Term::Var(Var::Len(view)))
                }
            }
            _ => None,
        }
    }

    fn term_range(&self, state: &State, term: Term) -> Interval {
        match term {
            Term::Var(var) => self.range(state, var),
            Term::Const(value) => Interval::exact(value),
        }
    }

    /// What `operand` can hold here.
    fn operand_range(&self, state: &State, operand: &Operand) -> Option<Interval> {
        if let Some(term) = self.term(operand) {
            return Some(self.term_range(state, term));
        }
        match operand {
            Operand::Copy(place) | Operand::Move(place) => {
                let full = type_range(self.types, place_type(self.body, self.types, place))?;
                Some(self.element_range(place).and_then(|range| range.meet(full)).unwrap_or(full))
            }
            _ => None,
        }
    }

    /// What an element read can hold: `xs[i]` of a followed list or of a
    /// shared view of one, or `*r` of a shared reference to such an element.
    fn element_range(&self, place: &Place) -> Option<Interval> {
        let list = match place.projection.as_slice() {
            [Projection::Index(_) | Projection::ConstIndex(_)] => match self.lists.stores.contains_key(&place.local) {
                true => place.local,
                false => *self.lists.views.get(&place.local)?,
            },
            // Every list the reference may point into: the hull of theirs.
            [Projection::Deref] => {
                let mut hull: Option<Interval> = None;
                for list in self.lists.items.get(&place.local)? {
                    let range = *self.elements.get(list)?;
                    hull = Some(hull.map_or(range, |hull| hull.hull(range)));
                }
                return hull;
            }
            _ => return None,
        };
        self.elements.get(&list).copied()
    }

    fn operand_ty(&self, operand: &Operand) -> Option<Ty> {
        match operand {
            Operand::Const(Const::Int { ty, .. }) => Some(*ty),
            Operand::Copy(place) | Operand::Move(place) => Some(place_type(self.body, self.types, place)),
            _ => None,
        }
    }

    /// The tightest `c` with `a <= b + c`: through known relations, then the
    /// ends of the last value's interval.
    fn le_bound(&self, state: &State, a: Term, b: Term) -> Option<i128> {
        // `a <= u + d` for each `u` reachable from `a`, and `w <= b + e` for
        // each `w` that reaches `b`; then `a <= b + d + (u.hi - w.lo) + e`,
        // or `d + e` through one value.
        let ends = |term: Term, forward: bool| -> Vec<(Term, i128)> {
            match term {
                Term::Var(var) => shortest_paths(state, var, forward)
                    .into_iter()
                    .map(|(var, d)| (Term::Var(var), d))
                    .collect(),
                Term::Const(_) => vec![(term, 0)],
            }
        };
        let mut bound: Option<i128> = None;
        for (u, d) in ends(a, true) {
            for (w, e) in ends(b, false) {
                let between = if u == w {
                    Some(0)
                } else {
                    self.term_range(state, u).hi.checked_sub(self.term_range(state, w).lo)
                };
                if let Some(c) = between.and_then(|x| x.checked_add(d)).and_then(|x| x.checked_add(e)) {
                    bound = Some(bound.map_or(c, |old| old.min(c)));
                }
            }
        }
        bound
    }

    /// Whether the comparison is known true or false here.
    fn decide(&self, state: &State, op: BinOp, lhs: Term, rhs: Term) -> Option<bool> {
        let le = |a, b, c: i128| self.le_bound(state, a, b).is_some_and(|bound| bound <= c);
        match op {
            BinOp::Lt if le(lhs, rhs, -1) => Some(true),
            BinOp::Lt if le(rhs, lhs, 0) => Some(false),
            BinOp::Le if le(lhs, rhs, 0) => Some(true),
            BinOp::Le if le(rhs, lhs, -1) => Some(false),
            BinOp::Gt => self.decide(state, BinOp::Lt, rhs, lhs),
            BinOp::Ge => self.decide(state, BinOp::Le, rhs, lhs),
            BinOp::Eq if le(lhs, rhs, 0) && le(rhs, lhs, 0) => Some(true),
            BinOp::Eq if le(lhs, rhs, -1) || le(rhs, lhs, -1) => Some(false),
            BinOp::Ne => self.decide(state, BinOp::Eq, lhs, rhs).map(|equal| !equal),
            _ => None,
        }
    }

    // -----------------------------------------------------------------------
    // Transfer.

    /// Forget every view `place` can change when written, moved or lent.
    fn forget_views(&self, state: &mut State, place: &Place) {
        for (index, view) in self.views.iter().enumerate() {
            if overlaps(place, &view.place) {
                state.forget(Var::Len(index));
            }
        }
    }

    fn forget_moved(&self, state: &mut State, operand: &Operand) {
        if let Operand::Move(place) = operand {
            self.forget_views(state, place);
        }
    }

    /// Record `a <= b + c`, narrowing both intervals; `None` when that cannot
    /// hold, so the path is not taken.
    fn assume_le(&self, state: &mut State, a: Term, b: Term, c: i128) -> Option<()> {
        let (ra, rb) = (self.term_range(state, a), self.term_range(state, b));
        if let Term::Var(var) = a {
            let hi = rb.hi.checked_add(c).unwrap_or(i128::MAX);
            let narrowed = ra.meet(Interval { lo: i128::MIN, hi })?;
            state.ranges.insert(var, narrowed);
        }
        if let Term::Var(var) = b {
            let lo = ra.lo.checked_sub(c).unwrap_or(i128::MIN);
            let narrowed = rb.meet(Interval { lo, hi: i128::MAX })?;
            state.ranges.insert(var, narrowed);
        }
        if let (Term::Var(x), Term::Var(y)) = (a, b) {
            if x == y {
                return (c >= 0).then_some(());
            }
            let entry = state.rels.entry((x, y)).or_insert(c);
            *entry = (*entry).min(c);
        }
        if let (Term::Const(x), Term::Const(y)) = (a, b) {
            return (x <= y.checked_add(c)?).then_some(());
        }
        Some(())
    }

    fn assume_ne(&self, state: &mut State, a: Term, b: Term) -> Option<()> {
        for (var, value) in [(a, b), (b, a)] {
            let (Term::Var(var), Term::Const(value)) = (var, value) else { continue };
            let mut range = self.range(state, var);
            if range.lo == value {
                range.lo = value.checked_add(1)?;
            }
            if range.hi == value {
                range.hi = value.checked_sub(1)?;
            }
            if range.lo > range.hi {
                return None;
            }
            state.ranges.insert(var, range);
        }
        if let (Term::Const(x), Term::Const(y)) = (a, b) {
            return (x != y).then_some(());
        }
        Some(())
    }

    /// Assume the comparison is `truth`.
    fn assume(&self, state: &mut State, op: BinOp, a: Term, b: Term, truth: bool) -> Option<()> {
        match (op, truth) {
            (BinOp::Lt, true) | (BinOp::Ge, false) => self.assume_le(state, a, b, -1),
            (BinOp::Lt, false) | (BinOp::Ge, true) => self.assume_le(state, b, a, 0),
            (BinOp::Le, true) | (BinOp::Gt, false) => self.assume_le(state, a, b, 0),
            (BinOp::Le, false) | (BinOp::Gt, true) => self.assume_le(state, b, a, -1),
            (BinOp::Eq, true) | (BinOp::Ne, false) => {
                self.assume_le(state, a, b, 0)?;
                self.assume_le(state, b, a, 0)
            }
            (BinOp::Eq, false) | (BinOp::Ne, true) => self.assume_ne(state, a, b),
            _ => Some(()),
        }
    }

    fn assume_bool(&self, state: &mut State, local: LocalId, truth: bool) -> Option<()> {
        match state.bools.get(&local).copied() {
            Some(BoolFact::Known(value)) => (value == truth).then_some(()),
            Some(BoolFact::Cmp { op, lhs, rhs }) => {
                self.assume(state, op, lhs, rhs, truth)?;
                state.bools.insert(local, BoolFact::Known(truth));
                Some(())
            }
            None => {
                state.bools.insert(local, BoolFact::Known(truth));
                Some(())
            }
        }
    }

    /// The fact a comparison gives its `bool`.
    fn compare(&self, state: &State, op: BinOp, lhs: &Operand, rhs: &Operand) -> Option<BoolFact> {
        if !op.is_comparison() || matches!(op, BinOp::Is | BinOp::IsNot) {
            return None;
        }
        if let (Some(a), Some(b)) = (self.term(lhs), self.term(rhs)) {
            return Some(match self.decide(state, op, a, b) {
                Some(value) => BoolFact::Known(value),
                None => BoolFact::Cmp { op, lhs: a, rhs: b },
            });
        }
        // A side the facts cannot name is still bounded by its type.
        let (a, b) = (self.operand_range(state, lhs)?, self.operand_range(state, rhs)?);
        let known = match op {
            BinOp::Lt => (a.hi < b.lo).then_some(true).or((a.lo >= b.hi).then_some(false)),
            BinOp::Le => (a.hi <= b.lo).then_some(true).or((a.lo > b.hi).then_some(false)),
            BinOp::Gt => (a.lo > b.hi).then_some(true).or((a.hi <= b.lo).then_some(false)),
            BinOp::Ge => (a.lo >= b.hi).then_some(true).or((a.hi < b.lo).then_some(false)),
            BinOp::Eq => (a.meet(b).is_none()).then_some(false),
            BinOp::Ne => (a.meet(b).is_none()).then_some(true),
            _ => None,
        };
        known.map(BoolFact::Known)
    }

    /// The value `rvalue` gives an integer local of type `ty`, and the
    /// relations to the local it gives: `(other, c_to, c_from)` for
    /// `x <= other + c_to` and `other <= x + c_from`.
    fn value(&self, state: &State, rvalue: &Rvalue, ty: Ty) -> (Option<Interval>, Vec<(Var, i128, i128)>) {
        let Some(full) = type_range(self.types, ty) else { return (None, Vec::new()) };
        let var_of = |operand: &Operand| match self.term(operand) {
            Some(Term::Var(var)) => Some(var),
            _ => None,
        };
        match rvalue {
            Rvalue::Use(operand) => {
                let range = self.operand_range(state, operand).and_then(|r| r.meet(full));
                let rels = var_of(operand).map(|var| vec![(var, 0, 0)]).unwrap_or_default();
                (range, rels)
            }
            Rvalue::Cast { kind: CastKind::Numeric | CastKind::Widen, operand, .. } => {
                let source = self.operand_range(state, operand);
                match source {
                    // The value is kept exactly.
                    Some(source) if source.within(full) => {
                        let rels = var_of(operand).map(|var| vec![(var, 0, 0)]).unwrap_or_default();
                        (Some(source), rels)
                    }
                    // A conversion of a non-negative integer keeps it or
                    // wraps it lower, so the result is at most the source.
                    Some(source) if source.lo >= 0 && self.operand_ty(operand).is_some_and(|ty| type_range(self.types, ty).is_some()) => {
                        let rels = var_of(operand).map(|var| vec![(var, 0, i128::MAX)]).unwrap_or_default();
                        (None, rels)
                    }
                    _ => (None, Vec::new()),
                }
            }
            Rvalue::BinaryOp { op, lhs, rhs } => self.arithmetic_value(state, *op, lhs, rhs, full, false),
            Rvalue::UnaryOp { op, operand } => {
                let Some(a) = self.operand_range(state, operand) else { return (None, Vec::new()) };
                let exact = match op {
                    UnOp::Neg => a.lo.checked_neg().zip(a.hi.checked_neg()).map(|(hi, lo)| Interval { lo, hi }),
                    // `!x` is `-x - 1` in two's complement, `MAX - x` unsigned.
                    UnOp::BitNot if full.lo < 0 => Some(Interval { lo: !a.hi, hi: !a.lo }),
                    UnOp::BitNot => Some(Interval { lo: full.hi - a.hi, hi: full.hi - a.lo }),
                    UnOp::Not => None,
                };
                (exact.filter(|r| r.within(full)), Vec::new())
            }
            _ => (None, Vec::new()),
        }
    }

    /// `lhs op rhs` into a local of range `full`. `passed`: the operation's
    /// overflow check has passed, so the exact result is the value.
    fn arithmetic_value(
        &self,
        state: &State,
        op: BinOp,
        lhs: &Operand,
        rhs: &Operand,
        full: Interval,
        passed: bool,
    ) -> (Option<Interval>, Vec<(Var, i128, i128)>) {
        let (Some(a), Some(b)) = (self.operand_range(state, lhs), self.operand_range(state, rhs)) else {
            return (None, Vec::new());
        };
        let exact = self.exact_result(state, op, lhs, rhs);
        let range = match exact {
            Some(exact) if passed => exact.meet(full),
            Some(exact) if exact.within(full) => Some(exact),
            _ => None,
        };
        // Relations hold only when the value is the exact result.
        let mut rels = Vec::new();
        if range.is_some() && exact.is_some() {
            let var = |operand: &Operand| match self.term(operand) {
                Some(Term::Var(var)) => Some(var),
                _ => None,
            };
            // `i128::MAX` stands for no relation that way.
            let neg = |x: i128| x.checked_neg().unwrap_or(i128::MAX);
            match op {
                BinOp::Add => {
                    if let Some(v) = var(lhs) {
                        rels.push((v, b.hi, neg(b.lo)));
                    }
                    if let Some(v) = var(rhs) {
                        rels.push((v, a.hi, neg(a.lo)));
                    }
                }
                BinOp::Sub => {
                    if let Some(v) = var(lhs) {
                        rels.push((v, neg(b.lo), b.hi));
                    }
                }
                _ => {}
            }
        }
        (range, rels)
    }

    /// Every exact result `lhs op rhs` can have here: the operands' ranges,
    /// and for `a - b` what is known between them (`a <= b + c` makes it at
    /// most `c`, `b <= a + d` at least `-d`).
    fn exact_result(&self, state: &State, op: BinOp, lhs: &Operand, rhs: &Operand) -> Option<Interval> {
        let (a, b) = (self.operand_range(state, lhs)?, self.operand_range(state, rhs)?);
        let width = self
            .operand_ty(lhs)
            .and_then(|ty| ember_types::bit_width(self.types, representation(self.types, ty)))
            .unwrap_or(0) as u32;
        let mut exact = arithmetic(op, a, b, width)?;
        if op == BinOp::Sub {
            if let (Some(ta), Some(tb)) = (self.term(lhs), self.term(rhs)) {
                if let Some(c) = self.le_bound(state, ta, tb) {
                    exact.hi = exact.hi.min(c);
                }
                if let Some(d) = self.le_bound(state, tb, ta) {
                    exact.lo = exact.lo.max(d.saturating_neg());
                }
            }
        }
        (exact.lo <= exact.hi).then_some(exact)
    }

    /// Give tracked local `local` a new value. A relation to the local's own
    /// old value (`i = i + 1`) moves every relation it had by the offset.
    fn set_local(&self, state: &mut State, local: LocalId, range: Option<Interval>, rels: Vec<(Var, i128, i128)>) {
        let me = Var::Local(local);
        let full = self.var_range(me);
        let mut moved: Vec<((Var, Var), i128)> = Vec::new();
        for (other, c_to, c_from) in &rels {
            if *other != me {
                continue;
            }
            // new <= old + c_to and old <= new + c_from.
            for ((a, b), c) in &state.rels {
                if *a == me && *b != me && *c_to != i128::MAX {
                    if let Some(c) = c.checked_add(*c_to) {
                        moved.push(((me, *b), c));
                    }
                }
                if *b == me && *a != me && *c_from != i128::MAX {
                    if let Some(c) = c.checked_add(*c_from) {
                        moved.push(((*a, me), c));
                    }
                }
            }
        }
        state.forget(me);
        state.bools.remove(&local);
        if let Some(range) = range.and_then(|range| range.meet(full)) {
            if range != full {
                state.ranges.insert(me, range);
            }
        }
        for (key, c) in moved {
            state.rels.insert(key, c);
        }
        for (other, c_to, c_from) in rels {
            if other == me {
                continue;
            }
            // `i128::MAX` stands for no relation that way.
            if c_to != i128::MAX {
                state.rels.insert((me, other), c_to);
            }
            if c_from != i128::MAX {
                state.rels.insert((other, me), c_from);
            }
        }
    }

    /// One statement's effect. `passed`: when the statement is a checked
    /// operation, its check is known to pass after it (the next terminator).
    fn step(&self, state: &mut State, stmt: &Stmt, passed: bool) {
        match &stmt.kind {
            StmtKind::Assign { place, rvalue } => {
                self.forget_views(state, place);
                for operand in rvalue_operands(rvalue) {
                    self.forget_moved(state, operand);
                }
                if let Rvalue::Ref { place: target, mutable: true } = rvalue {
                    self.forget_views(state, target);
                }
                if !place.projection.is_empty() {
                    return;
                }
                let local = place.local;
                if self.tracked[local.0 as usize] {
                    let (range, rels) = self.value(state, rvalue, self.body.local(local).ty);
                    self.set_local(state, local, range, rels);
                } else if matches!(self.types.kind(self.body.local(local).ty), TyKind::Bool) {
                    let fact = match rvalue {
                        Rvalue::BinaryOp { op, lhs, rhs } => self.compare(state, *op, lhs, rhs),
                        Rvalue::Use(Operand::Const(Const::Bool(value))) => Some(BoolFact::Known(*value)),
                        Rvalue::Use(Operand::Copy(from) | Operand::Move(from)) if from.projection.is_empty() => {
                            state.bools.get(&from.local).copied()
                        }
                        Rvalue::UnaryOp { op: UnOp::Not, operand: Operand::Copy(from) | Operand::Move(from) }
                            if from.projection.is_empty() =>
                        {
                            state.bools.get(&from.local).copied().map(negate)
                        }
                        _ => None,
                    };
                    self.forget_bool(state, local);
                    if let Some(fact) = fact {
                        state.bools.insert(local, fact);
                    }
                } else {
                    self.forget_bool(state, local);
                }
            }
            StmtKind::CheckedBinaryOp { dest, overflow, op, lhs, rhs } => {
                self.forget_views(state, dest);
                self.forget_views(state, overflow);
                let (Some(a), Some(b)) = (self.operand_range(state, lhs), self.operand_range(state, rhs)) else {
                    self.forget_place(state, dest);
                    self.forget_place(state, overflow);
                    return;
                };
                let dest_ty = place_type(self.body, self.types, dest);
                let exact = self.exact_result(state, *op, lhs, rhs);
                let fits = cannot_overflow(self.types, *op, a, b, exact, dest_ty);
                if dest.projection.is_empty() && self.tracked[dest.local.0 as usize] {
                    let full = self.var_range(Var::Local(dest.local));
                    let (range, rels) = self.arithmetic_value(state, *op, lhs, rhs, full, passed || fits);
                    self.set_local(state, dest.local, range, rels);
                } else {
                    self.forget_place(state, dest);
                }
                self.forget_place(state, overflow);
                if fits && overflow.projection.is_empty() {
                    state.bools.insert(overflow.local, BoolFact::Known(false));
                }
            }
            StmtKind::Drop { place, .. } => {
                self.forget_views(state, place);
                self.forget_place(state, place);
            }
            StmtKind::StorageLive(local) | StmtKind::StorageDead(local) => {
                self.forget_views(state, &Place::local(*local));
                self.forget_place(state, &Place::local(*local));
            }
            StmtKind::BeginAccess { .. }
            | StmtKind::BeginAccessTransfer { .. }
            | StmtKind::EndAccess { .. }
            | StmtKind::EndAccessTransfer { .. }
            | StmtKind::Nop => {}
        }
    }

    fn forget_bool(&self, state: &mut State, local: LocalId) {
        state.bools.remove(&local);
    }

    fn forget_place(&self, state: &mut State, place: &Place) {
        if place.projection.is_empty() {
            if self.tracked[place.local.0 as usize] {
                state.forget(Var::Local(place.local));
            }
            state.bools.remove(&place.local);
        }
    }

    /// The states the terminator of `block` passes to each successor.
    fn edges(&self, block: usize, mut state: State) -> Vec<(usize, State)> {
        let data = &self.body.blocks[block];
        match &data.terminator {
            Terminator::Goto(target) => vec![(target.0 as usize, state)],
            Terminator::SwitchInt { discr, targets, otherwise } => {
                let mut out = Vec::new();
                let local = match discr {
                    Operand::Copy(place) | Operand::Move(place) if place.projection.is_empty() => Some(place.local),
                    _ => None,
                };
                let is_bool = local.is_some_and(|l| matches!(self.types.kind(self.body.local(l).ty), TyKind::Bool));
                let term = self.term(discr);
                for (value, target) in targets {
                    let mut next = state.clone();
                    let ok = if is_bool {
                        self.assume_bool(&mut next, local.expect("a bool discriminant is a local"), *value != 0)
                    } else if let Some(term) = term {
                        self.assume(&mut next, BinOp::Eq, term, Term::Const(*value), true)
                    } else {
                        Some(())
                    };
                    if ok.is_some() {
                        out.push((target.0 as usize, next));
                    }
                }
                // A `bool` switch with one listed value: the other one.
                let ok = if is_bool && targets.len() == 1 {
                    self.assume_bool(&mut state, local.expect("a bool discriminant is a local"), targets[0].0 == 0)
                } else if let Some(term) = term.filter(|_| !is_bool) {
                    targets
                        .iter()
                        .try_for_each(|(value, _)| self.assume(&mut state, BinOp::Ne, term, Term::Const(*value), true))
                } else {
                    Some(())
                };
                if ok.is_some() {
                    out.push((otherwise.0 as usize, state));
                }
                out
            }
            Terminator::Call { func, args, dest, next } => {
                // A function that returns one of its parameters unchanged
                // gives one of those arguments.
                let returned = match func {
                    FuncRef::Direct { symbol, .. } => self.returns.get(symbol).and_then(|params| {
                        let ranges: Option<Vec<Interval>> =
                            params.iter().map(|&p| args.get(p).and_then(|arg| self.operand_range(&state, arg))).collect();
                        ranges?.into_iter().reduce(Interval::hull)
                    }),
                    _ => None,
                };
                for arg in args {
                    self.forget_moved(&mut state, arg);
                }
                // A call that only computes from its arguments changes no list.
                let computes = matches!(func, FuncRef::Builtin { which, .. } if computes_only(which));
                for (index, view) in self.views.iter().enumerate() {
                    if view.lent && !computes {
                        state.forget(Var::Len(index));
                    }
                }
                self.forget_views(&mut state, dest);
                self.forget_place(&mut state, dest);
                if dest.projection.is_empty() {
                    let local = dest.local;
                    match func {
                        FuncRef::Builtin { which, .. } if is_len(which) && self.tracked[local.0 as usize] => {
                            if let Some(Operand::Copy(view) | Operand::Move(view)) = args.first() {
                                if let Some(view) = self.view_of(view) {
                                    let len = Var::Len(view);
                                    let range = Some(self.range(&state, len));
                                    self.set_local(&mut state, local, range, vec![(len, 0, 0)]);
                                }
                            }
                        }
                        FuncRef::Builtin { which: Builtin::TotalLess, .. } if args.len() == 2 => {
                            if let Some(fact) = self.compare(&state, BinOp::Lt, &args[0], &args[1]) {
                                state.bools.insert(local, fact);
                            }
                        }
                        // `[CTL-3b]` — the value at `index` of a count of the
                        // values from `start` towards `stop` by a positive
                        // `step`: at most `stop - 1`, at least `start`.
                        FuncRef::Builtin { which: Builtin::RangeNth, .. } if self.tracked[local.0 as usize] => {
                            if let Some((range, rels)) = self.range_nth_value(&state, args) {
                                self.set_local(&mut state, local, Some(range), rels);
                            }
                        }
                        FuncRef::Direct { .. } if returned.is_some() && self.tracked[local.0 as usize] => {
                            self.set_local(&mut state, local, returned, Vec::new());
                        }
                        _ => {}
                    }
                }
                vec![(next.0 as usize, state)]
            }
            Terminator::Assert { cond, expected, next, .. } => {
                if let Operand::Copy(place) | Operand::Move(place) = cond {
                    if place.projection.is_empty() && self.assume_bool(&mut state, place.local, *expected).is_none() {
                        return Vec::new();
                    }
                }
                vec![(next.0 as usize, state)]
            }
            Terminator::Return | Terminator::Unreachable => Vec::new(),
        }
    }

    /// The state after `block`'s statements, before its terminator.
    fn exit_state(&self, block: usize, mut state: State) -> State {
        let data = &self.body.blocks[block];
        for (index, stmt) in data.stmts.iter().enumerate() {
            let passed = index + 1 == data.stmts.len() && checks_this(&data.terminator, stmt);
            self.step(&mut state, stmt, passed);
        }
        state
    }

    /// The facts, widening with thresholds; if that does not settle within
    /// the rounds allowed, without them, so a body never loses facts it had.
    fn solve(&mut self) -> Option<()> {
        self.solve_with(true).or_else(|| self.solve_with(false))
    }

    /// Per loop header, the thresholds its widening tries: every integer
    /// constant the loop holds and one either side of it, at most
    /// `MAX_THRESHOLDS` of them, nearest zero first.
    fn thresholds(&self, order: &[usize], headers: &HashSet<usize>) -> HashMap<usize, Vec<i128>> {
        let mut out = HashMap::new();
        for (header, inside) in natural_loops(self.body, order, headers) {
            let mut values = BTreeSet::new();
            let mut cases: Vec<i128> = Vec::new();
            let mut add = |operand: &Operand| {
                if let Operand::Const(Const::Int { value, ty }) = operand
                    && let Some(value) = constant(self.types, *value, *ty)
                {
                    values.extend([value.saturating_sub(1), value, value.saturating_add(1)]);
                }
            };
            for &block in &inside {
                let data = &self.body.blocks[block];
                for stmt in &data.stmts {
                    match &stmt.kind {
                        StmtKind::Assign { rvalue, .. } => match rvalue {
                            Rvalue::Use(a) | Rvalue::UnaryOp { operand: a, .. } | Rvalue::Cast { operand: a, .. } => add(a),
                            Rvalue::BinaryOp { lhs, rhs, .. } => {
                                add(lhs);
                                add(rhs);
                            }
                            Rvalue::Aggregate { operands, .. } => operands.iter().for_each(&mut add),
                            Rvalue::Repeat { value, .. } => add(value),
                            Rvalue::Discriminant(_) | Rvalue::Ref { .. } => {}
                        },
                        StmtKind::CheckedBinaryOp { lhs, rhs, .. } => {
                            add(lhs);
                            add(rhs);
                        }
                        _ => {}
                    }
                }
                match &data.terminator {
                    Terminator::Call { args, .. } => args.iter().for_each(&mut add),
                    Terminator::SwitchInt { targets, .. } => {
                        for (value, _) in targets {
                            cases.extend([value.saturating_sub(1), *value, value.saturating_add(1)]);
                        }
                    }
                    _ => {}
                }
            }
            values.extend(cases);
            let mut values: Vec<i128> = values.into_iter().collect();
            values.sort_by_key(|value| value.unsigned_abs());
            values.truncate(MAX_THRESHOLDS);
            values.sort();
            out.insert(header, values);
        }
        out
    }

    fn solve_with(&mut self, use_thresholds: bool) -> Option<()> {
        let n = self.body.blocks.len();
        let order = reverse_postorder(self.body);
        let headers = loop_headers(self.body, &order);
        let loop_writes = loop_writes(self.body, &order, &headers);
        let thresholds = if use_thresholds { self.thresholds(&order, &headers) } else { HashMap::new() };
        let unchanged = HashSet::new();
        let none: Vec<i128> = Vec::new();
        let mut entry: Vec<Option<State>> = vec![None; n];
        entry[0] = Some(State::default());
        let mut visits = vec![0usize; n];
        let mut rounds = 0;
        loop {
            rounds += 1;
            if rounds > MAX_ROUNDS {
                return None;
            }
            let mut changed = false;
            for &block in &order {
                if self.branch_only(block, &headers) {
                    continue;
                }
                let Some(state) = entry[block].clone() else { continue };
                let exit = self.exit_state(block, state);
                for (target, incoming) in self.successors_through_branches(block, exit, &headers) {
                    let merged = match &entry[target] {
                        None => incoming,
                        Some(old) => {
                            let joined = old.join(&incoming);
                            if headers.contains(&target) && visits[target] > 1 {
                                old.widen(
                                    &joined,
                                    self,
                                    loop_writes.get(&target).unwrap_or(&unchanged),
                                    thresholds.get(&target).unwrap_or(&none),
                                )
                            } else {
                                joined
                            }
                        }
                    };
                    let merged = self.seeded(target, merged);
                    if entry[target].as_ref() != Some(&merged) {
                        entry[target] = Some(merged);
                        visits[target] += 1;
                        changed = true;
                    }
                }
            }
            if !changed {
                break;
            }
        }
        // Narrowing: recompute every entry from the settled ones, twice.
        for _ in 0..2 {
            let mut next: Vec<Option<State>> = vec![None; n];
            next[0] = Some(State::default());
            for &block in &order {
                if self.branch_only(block, &headers) {
                    continue;
                }
                let Some(state) = entry[block].clone() else { continue };
                let exit = self.exit_state(block, state);
                for (target, incoming) in self.successors_through_branches(block, exit, &headers) {
                    if target == 0 {
                        continue;
                    }
                    let joined = match &next[target] {
                        None => incoming,
                        Some(old) => old.join(&incoming),
                    };
                    next[target] = Some(self.seeded(target, joined));
                }
            }
            if entry[0].is_some() {
                next[0] = entry[0].clone();
            }
            entry = next;
        }
        self.entry = entry;
        Some(())
    }

    /// `state` with the ranges known at loop header `block` whatever the path.
    fn seeded(&self, block: usize, mut state: State) -> State {
        for (header, var, range) in &self.seeds {
            if *header == block {
                if let Some(narrowed) = self.range(&state, *var).meet(*range) {
                    state.ranges.insert(*var, narrowed);
                }
            }
        }
        state
    }

    /// The state just before statement `index` of `block`.
    fn state_before(&self, block: usize, index: usize) -> Option<State> {
        let data = &self.body.blocks[block];
        let mut state = self.entry.get(block)?.clone()?;
        for stmt in data.stmts.iter().take(index) {
            self.step(&mut state, stmt, false);
        }
        Some(state)
    }

    /// `[RNG-4]` — the range of each running total of a counted loop at its
    /// header. A local the loop changes only by one chain of checked `+` and
    /// `-` back to itself (`total = total + a + b - c`) changes each turn by
    /// the sum of its terms' ranges; the loop turns at most as many times as
    /// its limit less its counter's start allow; so at the header it lies
    /// between its value on entry and that plus every turn's change. The
    /// chain is checked, so no turn can wrap: an overflow ends the program
    /// before the next. Only a range within the total's type is kept.
    fn accumulator_bounds(&self) -> Vec<(usize, Var, Interval)> {
        let mut seeds = Vec::new();
        let headers = loop_headers(self.body, &reverse_postorder(self.body));
        for header in 0..self.body.blocks.len() {
            if self.entry[header].is_none() {
                continue;
            }
            let Some(shape) = crate::loop_version::counted_loop(self.body, self.types, header) else { continue };
            let mut inside: Vec<usize> = shape.region.clone();
            inside.push(header);
            // Every place a local is written in the loop: (block, statement).
            let mut writes: HashMap<LocalId, Vec<(usize, usize)>> = HashMap::new();
            let mut called: HashSet<LocalId> = HashSet::new();
            for &block in &inside {
                for (index, stmt) in self.body.blocks[block].stmts.iter().enumerate() {
                    let place = match &stmt.kind {
                        StmtKind::Assign { place, .. } => place,
                        StmtKind::CheckedBinaryOp { dest, .. } => dest,
                        StmtKind::StorageLive(local) | StmtKind::StorageDead(local) => {
                            called.insert(*local);
                            continue;
                        }
                        _ => continue,
                    };
                    writes.entry(place.local).or_default().push((block, index));
                }
                if let Terminator::Call { dest, .. } = &self.body.blocks[block].terminator {
                    called.insert(dest.local);
                }
            }
            let counter_writes = writes.get(&shape.counter).map_or(0, Vec::len);
            if writes.contains_key(&shape.limit)
                || called.contains(&shape.limit)
                || called.contains(&shape.counter)
                || counter_writes != 1
            {
                continue;
            }
            // The state on entry from outside the loop.
            let mut outside: Option<State> = None;
            for pred in 0..self.body.blocks.len() {
                if inside.contains(&pred) {
                    continue;
                }
                if self.branch_only(pred, &headers) {
                    continue;
                }
                let Some(state) = self.entry[pred].clone() else { continue };
                for (target, state) in self.successors_through_branches(pred, self.exit_state(pred, state), &headers) {
                    if target == header {
                        outside = Some(match outside {
                            None => state,
                            Some(old) => old.join(&state),
                        });
                    }
                }
            }
            let Some(outside) = outside else { continue };
            let (start, limit) = (self.range(&outside, Var::Local(shape.counter)), self.range(&outside, Var::Local(shape.limit)));
            let Some(turns) = limit.hi.checked_sub(start.lo).and_then(|n| n.checked_add(i128::from(shape.inclusive))) else {
                continue;
            };
            let turns = turns.max(0);
            for (&acc, places) in &writes {
                let [(block, index)] = places.as_slice() else { continue };
                if acc == shape.counter || acc == shape.limit || !self.tracked[acc.0 as usize] || called.contains(&acc) {
                    continue;
                }
                let Some(change) = self.chain_change(acc, *block, *index, &writes, 0) else { continue };
                let entry = self.range(&outside, Var::Local(acc));
                let low = turns.checked_mul(change.lo.min(0)).and_then(|d| entry.lo.checked_add(d));
                let high = turns.checked_mul(change.hi.max(0)).and_then(|d| entry.hi.checked_add(d));
                let (Some(lo), Some(hi)) = (low, high) else { continue };
                let bound = Interval { lo, hi };
                if bound.within(self.var_range(Var::Local(acc))) {
                    seeds.push((header, Var::Local(acc), bound));
                }
            }
        }
        seeds
    }

    /// The change one turn makes to `acc`, written at statement `index` of
    /// `block`: a checked `+` or `-` whose one side is `acc` itself or a
    /// local the loop sets once by such an operation from it.
    fn chain_change(
        &self,
        acc: LocalId,
        block: usize,
        index: usize,
        writes: &HashMap<LocalId, Vec<(usize, usize)>>,
        depth: usize,
    ) -> Option<Interval> {
        if depth > 16 {
            return None;
        }
        let StmtKind::CheckedBinaryOp { op: op @ (BinOp::Add | BinOp::Sub), lhs, rhs, .. } =
            &self.body.blocks[block].stmts[index].kind
        else {
            return None;
        };
        let before = self.state_before(block, index)?;
        // The side that carries the total, and the term the other adds.
        let carried = |operand: &Operand| -> Option<Interval> {
            let (Operand::Copy(place) | Operand::Move(place)) = operand else { return None };
            if !place.projection.is_empty() {
                return None;
            }
            if place.local == acc {
                return Some(Interval::exact(0));
            }
            let [(b, i)] = writes.get(&place.local)?.as_slice() else { return None };
            self.chain_change(acc, *b, *i, writes, depth + 1)
        };
        let term = |operand: &Operand| self.operand_range(&before, operand);
        let (so_far, added) = match (carried(lhs), op) {
            (Some(so_far), _) => (so_far, term(rhs)?),
            (None, BinOp::Add) => (carried(rhs)?, term(lhs)?),
            _ => return None,
        };
        let added = if *op == BinOp::Sub {
            Interval { lo: added.hi.checked_neg()?, hi: added.lo.checked_neg()? }
        } else {
            added
        };
        Some(Interval { lo: so_far.lo.checked_add(added.lo)?, hi: so_far.hi.checked_add(added.hi)? })
    }

    /// A block that only sets `bool`s and branches on one, outside any loop
    /// header: `a and b`, `a or b` and `not` join their paths in such a block.
    /// Each path through it is followed on its own, so what one path knew
    /// is not lost in the join before the branch that depends on it.
    fn branch_only(&self, block: usize, headers: &HashSet<usize>) -> bool {
        let data = &self.body.blocks[block];
        block != 0
            && !headers.contains(&block)
            && matches!(&data.terminator, Terminator::SwitchInt { discr: Operand::Copy(place) | Operand::Move(place), .. }
                if place.projection.is_empty())
            && data.stmts.iter().all(|stmt| match &stmt.kind {
                StmtKind::Assign { place, .. } => {
                    place.projection.is_empty() && matches!(self.types.kind(self.body.local(place.local).ty), TyKind::Bool)
                }
                StmtKind::StorageLive(_) | StmtKind::StorageDead(_) | StmtKind::Nop => true,
                _ => false,
            })
    }

    /// The branches every run reaching them takes one way, each with that
    /// way. A branch-only block has no state of its own (the solve follows
    /// the edges through it), so each is judged on the states of the edges
    /// reaching it.
    fn decided_branches(&self) -> Vec<(usize, ember_mir::BasicBlockId)> {
        let order = reverse_postorder(self.body);
        let headers = loop_headers(self.body, &order);
        let mut taken: BTreeMap<usize, BTreeSet<usize>> = BTreeMap::new();
        for block in 0..self.body.blocks.len() {
            let Some(state) = self.entry[block].clone() else { continue };
            let mut work = vec![(block, self.exit_state(block, state))];
            while let Some((at, exit)) = work.pop() {
                let branch = matches!(self.body.blocks[at].terminator, Terminator::SwitchInt { .. });
                if branch {
                    taken.entry(at).or_default();
                }
                for (target, state) in self.edges(at, exit) {
                    if branch {
                        taken.entry(at).or_default().insert(target);
                    }
                    if self.branch_only(target, &headers) {
                        work.push((target, self.exit_state(target, state)));
                    }
                }
            }
        }
        taken
            .into_iter()
            .filter_map(|(block, targets)| {
                let Terminator::SwitchInt { targets: ways, otherwise, .. } = &self.body.blocks[block].terminator else {
                    unreachable!("a branch ends in a switch")
                };
                let ways: BTreeSet<usize> = ways.iter().map(|(_, way)| way.0 as usize).chain([otherwise.0 as usize]).collect();
                let [way] = *targets.iter().collect::<Vec<_>>().as_slice() else { return None };
                (ways.len() > 1).then_some((block, ember_mir::BasicBlockId(*way as u32)))
            })
            .collect()
    }

    /// `edges(block)`, with each edge into a branch-only block followed
    /// through it on the edge's own state. The recursion ends at a loop
    /// header, since a cycle passes through one.
    fn successors_through_branches(&self, block: usize, exit: State, headers: &HashSet<usize>) -> Vec<(usize, State)> {
        let mut out = Vec::new();
        let mut work = self.edges(block, exit);
        while let Some((target, state)) = work.pop() {
            if self.branch_only(target, headers) {
                let exit = self.exit_state(target, state);
                work.extend(self.edges(target, exit));
            } else {
                out.push((target, state));
            }
        }
        out
    }

    /// Upper bounds of `index` at the end of `block`'s statements: its
    /// interval's top, and each `index <= v + c` for a tracked local `v`.
    pub(crate) fn upper_bounds(&self, block: usize, index: &Operand) -> Option<(i128, Vec<(LocalId, i128)>)> {
        let state = self.exit_state(block, self.entry.get(block)?.clone()?);
        let Some(Term::Var(var)) = self.term(index) else {
            return self.operand_range(&state, index).map(|range| (range.hi, Vec::new()));
        };
        let hi = self.range(&state, var).hi;
        let mut symbolic = Vec::new();
        for local in 0..self.body.locals.len() {
            let other = Var::Local(LocalId(local as u32));
            if !self.tracked[local] || other == var {
                continue;
            }
            if let Some(c) = self.le_bound_via_rels(&state, var, other) {
                symbolic.push((LocalId(local as u32), c));
            }
        }
        Some((hi, symbolic))
    }

    /// The tightest `c` with `a <= b + c` from relations alone.
    fn le_bound_via_rels(&self, state: &State, a: Var, b: Var) -> Option<i128> {
        shortest_paths(state, a, true).get(&b).copied()
    }
}

/// Forward: for every value `v` reachable from `start` through the
/// relations, the smallest `d` with `start <= v + d`. Backward: for every `v`
/// that reaches `start`, the smallest `d` with `v <= start + d`. Bellman-Ford,
/// one round per value at most; a negative cycle (a state no run reaches)
/// stops at that bound.
fn shortest_paths(state: &State, start: Var, forward: bool) -> BTreeMap<Var, i128> {
    let mut best: BTreeMap<Var, i128> = BTreeMap::from([(start, 0)]);
    let values: HashSet<Var> = state.rels.keys().flat_map(|(a, b)| [*a, *b]).collect();
    for _ in 0..values.len().max(1) {
        let mut changed = false;
        for ((a, b), c) in &state.rels {
            let (from, to) = if forward { (a, b) } else { (b, a) };
            let Some(d) = best.get(from).and_then(|d| d.checked_add(*c)) else { continue };
            if best.get(to).is_none_or(|old| d < *old) {
                best.insert(*to, d);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    best
}

fn negate(fact: BoolFact) -> BoolFact {
    match fact {
        BoolFact::Known(value) => BoolFact::Known(!value),
        BoolFact::Cmp { op, lhs, rhs } => {
            let op = match op {
                BinOp::Lt => BinOp::Ge,
                BinOp::Ge => BinOp::Lt,
                BinOp::Le => BinOp::Gt,
                BinOp::Gt => BinOp::Le,
                BinOp::Eq => BinOp::Ne,
                BinOp::Ne => BinOp::Eq,
                other => other,
            };
            BoolFact::Cmp { op, lhs, rhs }
        }
    }
}

/// A built-in that computes a value from its arguments and touches no list:
/// a stepped loop's count and value, a length, a total-order comparison.
fn computes_only(which: &Builtin) -> bool {
    matches!(
        which,
        Builtin::RangeCount | Builtin::RangeNth | Builtin::ArrayLen | Builtin::SpanLen | Builtin::StringLen | Builtin::TotalLess
    )
}

/// A list call that only reorders, removes or reads elements, or reserves room.
fn reorders_or_removes(which: &Builtin) -> bool {
    matches!(
        which,
        Builtin::ArraySort
            | Builtin::ArrayReverse
            | Builtin::ArrayClear
            | Builtin::ArrayPop { .. }
            | Builtin::ArrayRemove
            | Builtin::ArrayDrain
            | Builtin::ArrayReserve
            | Builtin::ArrayTruncate
            | Builtin::StringRemove
            | Builtin::StringTruncate
            | Builtin::ArraySwapRemove
            | Builtin::ArraySwap
            | Builtin::ArrayLen
            | Builtin::ArrayCapacity
    )
}

fn is_len(which: &Builtin) -> bool {
    matches!(which, Builtin::ArrayLen | Builtin::SpanLen | Builtin::StringLen)
}

/// Whether writing, moving or lending `place` can change the list `view`:
/// the same local, and one place holds the other, unless `place` is inside
/// an element.
fn overlaps(place: &Place, view: &Place) -> bool {
    if place.local != view.local {
        return false;
    }
    let n = view.projection.len().min(place.projection.len());
    if place.projection[..n] != view.projection[..n] {
        return false;
    }
    !matches!(place.projection.get(view.projection.len()), Some(Projection::Index(_) | Projection::ConstIndex(_)))
}

fn rvalue_operands(rvalue: &Rvalue) -> impl Iterator<Item = &Operand> {
    let operands: Vec<&Operand> = match rvalue {
        Rvalue::Use(value) | Rvalue::UnaryOp { operand: value, .. } | Rvalue::Cast { operand: value, .. } => vec![value],
        Rvalue::Repeat { value, .. } => vec![value],
        Rvalue::BinaryOp { lhs, rhs, .. } => vec![lhs, rhs],
        Rvalue::Aggregate { operands, .. } => operands.iter().collect(),
        Rvalue::Discriminant(_) | Rvalue::Ref { .. } => Vec::new(),
    };
    operands.into_iter()
}

fn for_each_place(stmt: &Stmt, f: &mut impl FnMut(&Place)) {
    let operand = |operand: &Operand, f: &mut dyn FnMut(&Place)| {
        if let Operand::Copy(place) | Operand::Move(place) = operand {
            f(place);
        }
    };
    match &stmt.kind {
        StmtKind::Assign { place, rvalue } => {
            f(place);
            for value in rvalue_operands(rvalue) {
                operand(value, f);
            }
            if let Rvalue::Ref { place, .. } | Rvalue::Discriminant(place) = rvalue {
                f(place);
            }
        }
        StmtKind::CheckedBinaryOp { dest, lhs, rhs, .. } => {
            f(dest);
            operand(lhs, f);
            operand(rhs, f);
        }
        _ => {}
    }
}

/// Whether `terminator` is the check of `stmt`, a checked operation.
fn checks_this(terminator: &Terminator, stmt: &Stmt) -> bool {
    let StmtKind::CheckedBinaryOp { overflow, .. } = &stmt.kind else { return false };
    matches!(terminator, Terminator::Assert { cond: Operand::Copy(cond), expected: false, .. } if cond == overflow)
}

/// Whether `a op b` into `ty` can never overflow: its exact result (`exact`)
/// always fits. `[TYP-28]`'s floor remainder never overflows; a division
/// only at `MIN / -1`. A left shift of a negative number is left checked.
fn cannot_overflow(types: &TypeTable, op: BinOp, a: Interval, b: Interval, exact: Option<Interval>, ty: Ty) -> bool {
    let Some(full) = type_range(types, representation(types, ty)) else { return false };
    match op {
        BinOp::FloorRem => true,
        BinOp::Div | BinOp::FloorDiv | BinOp::Rem => !(a.contains(full.lo) && full.lo < 0 && b.contains(-1)),
        BinOp::Shl if a.lo < 0 => false,
        _ => exact.is_some_and(|exact| exact.within(full)),
    }
}

pub(crate) fn reverse_postorder(body: &Body) -> Vec<usize> {
    let n = body.blocks.len();
    let mut seen = vec![false; n];
    let mut order = Vec::new();
    // Iterative depth-first search: (block, next successor to visit).
    let mut stack: Vec<(usize, usize)> = vec![(0, 0)];
    seen[0] = true;
    while let Some((block, next)) = stack.pop() {
        let successors = crate::long_access_lint::successors(&body.blocks[block].terminator);
        if let Some(&succ) = successors.get(next) {
            stack.push((block, next + 1));
            if !seen[succ] {
                seen[succ] = true;
                stack.push((succ, 0));
            }
        } else {
            order.push(block);
        }
    }
    order.reverse();
    order
}

/// Blocks entered by a back edge: a successor earlier in reverse postorder.
pub(crate) fn loop_headers(body: &Body, order: &[usize]) -> HashSet<usize> {
    let position: HashMap<usize, usize> = order.iter().enumerate().map(|(i, &b)| (b, i)).collect();
    let mut headers = HashSet::new();
    for &block in order {
        for succ in crate::long_access_lint::successors(&body.blocks[block].terminator) {
            if position.get(&succ).is_some_and(|&p| p <= position[&block]) {
                headers.insert(succ);
            }
        }
    }
    headers
}

/// Per loop header, the blocks of its loop: those that reach a back edge
/// into the header without passing it, and the header.
pub(crate) fn natural_loops(body: &Body, order: &[usize], headers: &HashSet<usize>) -> HashMap<usize, HashSet<usize>> {
    let position: HashMap<usize, usize> = order.iter().enumerate().map(|(i, &b)| (b, i)).collect();
    let mut preds: HashMap<usize, Vec<usize>> = HashMap::new();
    for &block in order {
        for succ in crate::long_access_lint::successors(&body.blocks[block].terminator) {
            preds.entry(succ).or_default().push(block);
        }
    }
    let mut blocks: HashMap<usize, HashSet<usize>> = HashMap::new();
    for &latch in order {
        for header in crate::long_access_lint::successors(&body.blocks[latch].terminator) {
            if !headers.contains(&header) || position.get(&header).is_none_or(|&p| p > position[&latch]) {
                continue;
            }
            let inside = blocks.entry(header).or_insert_with(|| HashSet::from([header]));
            let mut work = vec![latch];
            while let Some(block) = work.pop() {
                if inside.insert(block) {
                    work.extend(preds.get(&block).into_iter().flatten().copied());
                }
            }
        }
    }
    blocks
}

/// Per loop header, every local its loop writes.
fn loop_writes(body: &Body, order: &[usize], headers: &HashSet<usize>) -> HashMap<usize, HashSet<LocalId>> {
    natural_loops(body, order, headers)
        .into_iter()
        .map(|(header, inside)| {
            let mut writes = HashSet::new();
            for block in inside {
                let data = &body.blocks[block];
                for stmt in &data.stmts {
                    match &stmt.kind {
                        StmtKind::Assign { place, .. } => {
                            writes.insert(place.local);
                        }
                        StmtKind::CheckedBinaryOp { dest, overflow, .. } => {
                            writes.insert(dest.local);
                            writes.insert(overflow.local);
                        }
                        StmtKind::Drop { place, .. } => {
                            writes.insert(place.local);
                        }
                        StmtKind::StorageLive(local) | StmtKind::StorageDead(local) => {
                            writes.insert(*local);
                        }
                        _ => {}
                    }
                }
                if let Terminator::Call { dest, .. } = &data.terminator {
                    writes.insert(dest.local);
                }
            }
            (header, writes)
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Removing the checks.

fn remove_proven_checks(
    body: &mut Body,
    types: &TypeTable,
    common: &CommonTypes,
    returns: &HashMap<String, Vec<usize>>,
) -> (usize, usize) {
    // What to change, found on the unchanged body.
    struct Removal {
        block: usize,
        kind: CheckKind,
        /// The checked operation to make plain, with its operand ranges.
        checked: Option<(Interval, Interval)>,
    }
    let removals: Vec<Removal> = {
        let Some(analysis) = Analysis::run(body, types, common, returns) else { return (0, 0) };
        let mut removals = Vec::new();
        let reached: HashSet<usize> = reverse_postorder(body).into_iter().collect();
        for block in 0..body.blocks.len() {
            let data = &body.blocks[block];
            // A check in a block no run reaches (every way in contradicts the
            // facts) can never fail. Branch-only blocks keep no state; they
            // end in a switch, not a check.
            let Some(entry) = analysis.entry[block].clone() else {
                if reached.contains(&block) {
                    if let Terminator::Assert { msg, cond: Operand::Copy(cond), .. } = &data.terminator {
                        // A checked operation's own check goes with it, made
                        // plain over its operands' whole types.
                        let checked = match data.stmts.last().map(|stmt| &stmt.kind) {
                            Some(StmtKind::CheckedBinaryOp { overflow, lhs, rhs, .. }) if overflow == cond => {
                                let whole = |operand: &Operand| {
                                    analysis.operand_ty(operand).and_then(|ty| type_range(types, representation(types, ty)))
                                };
                                whole(lhs).zip(whole(rhs))
                            }
                            _ => None,
                        };
                        let kind = match msg {
                            AssertKind::Bounds { .. } => Some(CheckKind::Bounds),
                            AssertKind::DivisionByZero => Some(CheckKind::DivisionByZero),
                            AssertKind::ShiftTooLarge => Some(CheckKind::ShiftRange),
                            AssertKind::Overflow(_) | AssertKind::SignedDivisionOverflow if checked.is_some() => {
                                Some(CheckKind::Overflow)
                            }
                            _ => None,
                        };
                        if let Some(kind) = kind {
                            removals.push(Removal { block, kind, checked });
                        }
                    }
                }
                continue;
            };
            let Terminator::Assert { cond: Operand::Copy(cond), expected, msg, .. } = &data.terminator else {
                continue;
            };
            if !cond.projection.is_empty() {
                continue;
            }
            // The ranges before the last statement, for a checked operation.
            let mut before = entry;
            let count = data.stmts.len();
            for stmt in data.stmts.iter().take(count.saturating_sub(1)) {
                analysis.step(&mut before, stmt, false);
            }
            let mut after = before.clone();
            let checked = match data.stmts.last() {
                Some(stmt) => {
                    analysis.step(&mut after, stmt, false);
                    match &stmt.kind {
                        StmtKind::CheckedBinaryOp { overflow, lhs, rhs, .. } if overflow == cond => analysis
                            .operand_range(&before, lhs)
                            .zip(analysis.operand_range(&before, rhs)),
                        _ => None,
                    }
                }
                None => None,
            };
            if after.bools.get(&cond.local) != Some(&BoolFact::Known(*expected)) {
                continue;
            }
            let kind = match msg {
                AssertKind::Bounds { .. } => CheckKind::Bounds,
                AssertKind::Overflow(_) | AssertKind::SignedDivisionOverflow => CheckKind::Overflow,
                AssertKind::DivisionByZero => CheckKind::DivisionByZero,
                AssertKind::ShiftTooLarge => CheckKind::ShiftRange,
                // Explicit panics and dynamic borrow checks are not range
                // checks; they stay.
                _ => continue,
            };
            // A checked operation's own check goes with the operation made
            // plain; any other check's condition must not come from one.
            if matches!(kind, CheckKind::Overflow) && checked.is_none() {
                continue;
            }
            removals.push(Removal { block, kind, checked });
        }
        removals
    };
    // A branch whose test the facts decide goes one way.
    let folds = Analysis::run(body, types, common, returns).map_or_else(Vec::new, |analysis| analysis.decided_branches());
    for removal in &removals {
        let span = body.blocks[removal.block].terminator_span;
        let Terminator::Assert { next, cond, .. } = body.blocks[removal.block].terminator.clone() else {
            unreachable!("a removal is an assert")
        };
        body.blocks[removal.block].terminator = Terminator::Goto(next);
        body.removed_checks.push(RemovedCheck { span, kind: removal.kind, proof: CheckProof::RangeFacts });
        if let Some((a, b)) = removal.checked {
            make_plain(body, types, common, removal.block, a, b);
        } else if let Operand::Copy(cond) = cond {
            drop_unread_condition(body, removal.block, cond.local);
        }
    }
    for &(block, target) in &folds {
        body.blocks[block].terminator = Terminator::Goto(target);
    }
    if !folds.is_empty() {
        clear_dead_code(body);
    }
    (removals.len(), folds.len())
}

/// After branches are folded: the blocks no run reaches now are emptied (so
/// nothing they read stays alive), then the temporaries nothing reads, set
/// by a computation over locals alone, are removed, in rounds.
fn clear_dead_code(body: &mut Body) {
    let mut reached = vec![false; body.blocks.len()];
    let mut work = vec![0usize];
    while let Some(block) = work.pop() {
        if block >= reached.len() || reached[block] {
            continue;
        }
        reached[block] = true;
        match &body.blocks[block].terminator {
            Terminator::Goto(next) | Terminator::Call { next, .. } | Terminator::Assert { next, .. } => work.push(next.0 as usize),
            Terminator::SwitchInt { targets, otherwise, .. } => {
                work.extend(targets.iter().map(|(_, target)| target.0 as usize));
                work.push(otherwise.0 as usize);
            }
            Terminator::Return | Terminator::Unreachable => {}
        }
    }
    for (block, data) in body.blocks.iter_mut().enumerate() {
        if !reached[block] {
            data.stmts.clear();
            data.terminator = Terminator::Unreachable;
        }
    }
    loop {
        let mut read = vec![false; body.locals.len()];
        for data in &body.blocks {
            data.stmts.iter().for_each(|stmt| crate::strength_reduce::stmt_reads(stmt, &mut read));
            crate::strength_reduce::terminator_reads(&data.terminator, &mut read);
        }
        let plain = |operand: &Operand| match operand {
            Operand::Const(_) => true,
            Operand::Copy(place) | Operand::Move(place) => place.projection.is_empty(),
        };
        let mut removed = false;
        for data in &mut body.blocks {
            for stmt in &mut data.stmts {
                let StmtKind::Assign { place, rvalue } = &stmt.kind else { continue };
                let pure = match rvalue {
                    Rvalue::Use(operand) | Rvalue::Cast { operand, .. } | Rvalue::UnaryOp { operand, .. } => plain(operand),
                    Rvalue::BinaryOp { lhs, rhs, .. } => plain(lhs) && plain(rhs),
                    _ => false,
                };
                let local = place.local.0 as usize;
                if pure && place.projection.is_empty() && !read[local] && body.locals[local].kind == ember_mir::LocalKind::Temp {
                    stmt.kind = StmtKind::Nop;
                    removed = true;
                }
            }
        }
        if !removed {
            return;
        }
    }
}

/// Replace the checked operation ending `block`, whose check is gone, with the
/// plain operation that means the same. `a` and `b` are its operands' ranges.
fn make_plain(body: &mut Body, types: &TypeTable, common: &CommonTypes, block: usize, a: Interval, b: Interval) {
    let stmt = body.blocks[block].stmts.pop().expect("a checked operation ends the block");
    let span = stmt.span;
    let StmtKind::CheckedBinaryOp { dest, op, lhs, rhs, .. } = stmt.kind else {
        unreachable!("the block's last statement is its checked operation")
    };
    let ty = representation(types, place_type(body, types, &dest));
    let signed = ember_types::is_signed(types, ty) == Some(true);
    let assign = |rvalue: Rvalue| Stmt::new(StmtKind::Assign { place: dest.clone(), rvalue }, span);
    let binary = |op: BinOp, lhs: Operand, rhs: Operand| Rvalue::BinaryOp { op, lhs, rhs };
    let stmts = match op {
        BinOp::FloorDiv | BinOp::FloorRem if !signed => {
            let op = if op == BinOp::FloorDiv { BinOp::Div } else { BinOp::Rem };
            vec![assign(binary(op, lhs, rhs))]
        }
        BinOp::FloorDiv | BinOp::FloorRem => {
            let power = (b.lo == b.hi && b.lo > 0 && b.lo & (b.lo - 1) == 0).then(|| b.lo.trailing_zeros());
            let int = |value: i128| Operand::Const(Const::Int { value: value as u128, ty });
            match (op, power) {
                // Two's complement: `>> k` is floor division by `2^k`, and
                // `& (2^k - 1)` the floor remainder, for either sign.
                (BinOp::FloorDiv, Some(k)) => vec![assign(binary(BinOp::Shr, lhs, int(i128::from(k))))],
                (BinOp::FloorRem, Some(_)) => vec![assign(binary(BinOp::BitAnd, lhs, int(b.lo - 1)))],
                // Both sides non-negative: C's unsigned operators, which the
                // C compiler need not correct for a sign.
                // Both sides non-negative: C's truncating operators give the
                // floor. By a constant, a C compiler multiplies by a magic
                // number: unsigned when that number fits the width (two
                // instructions), else C's signed operator, as hand-written C
                // gets (an unsigned divisor such as 7 costs a five-instruction
                // correction), in the operands' own width either way, so
                // `[SIMD-5]` sees the division it is (ODR-088). By a variable,
                // which no vector unit divides by, unsigned: 32-bit when both
                // fit, which every processor divides several times faster.
                _ if a.lo >= 0 && b.lo > 0 && b.lo == b.hi && !unsigned_magic_fits(b.lo as u128, width_of(types, ty)) => {
                    let plain = if op == BinOp::FloorDiv { BinOp::Div } else { BinOp::Rem };
                    vec![assign(binary(plain, lhs, rhs))]
                }
                _ if a.lo >= 0 && b.lo > 0 => {
                    let fits_32 = b.lo != b.hi && a.hi <= i128::from(u32::MAX) && b.hi <= i128::from(u32::MAX)
                        && ember_types::bit_width(types, ty).is_some_and(|w| w > 32);
                    let unsigned = if fits_32 { common.u32 } else { unsigned_of(types, common, ty) };
                    let temp = |body: &mut Body| {
                        body.locals.push(ember_mir::LocalDecl {
                            ty: unsigned,
                            kind: ember_mir::LocalKind::Temp,
                            name: None,
                            span,
                        });
                        Place::local(LocalId(body.locals.len() as u32 - 1))
                    };
                    let cast = |operand: Operand, to: Ty| Rvalue::Cast { kind: CastKind::Numeric, operand, to };
                    let set = |place: &Place, rvalue: Rvalue| Stmt::new(StmtKind::Assign { place: place.clone(), rvalue }, span);
                    let plain = if op == BinOp::FloorDiv { BinOp::Div } else { BinOp::Rem };
                    let mut stmts = Vec::new();
                    // An operand already a constant stays one, so the C compiler
                    // and the loop passes see a constant divisor.
                    let mut unsigned_operand = |operand: Operand, stmts: &mut Vec<Stmt>| match operand {
                        Operand::Const(Const::Int { value, .. }) => Operand::Const(Const::Int { value, ty: unsigned }),
                        other => {
                            let place = temp(body);
                            stmts.push(set(&place, cast(other, unsigned)));
                            Operand::Copy(place)
                        }
                    };
                    let x = unsigned_operand(lhs, &mut stmts);
                    let y = unsigned_operand(rhs, &mut stmts);
                    let r = temp(body);
                    stmts.push(set(&r, binary(plain, x, y)));
                    stmts.push(assign(cast(Operand::Copy(r), place_type(body, types, &dest))));
                    stmts
                }
                // A positive divisor, the dividend of either sign: C's `%`
                // rounds toward zero, so a negative remainder is one divisor
                // short, and the quotient one too many. The sign bit of the
                // remainder, `r >> (w - 1)` (0 or -1), corrects both without a
                // branch: `r + (s & b)` and `q + s`. Neither can overflow.
                _ if b.lo > 0 => {
                    let width = ember_types::bit_width(types, ty).expect("a signed integer has a width") as i128;
                    let as_copy = |operand: Operand| match operand {
                        Operand::Move(place) => Operand::Copy(place),
                        other => other,
                    };
                    let (lhs, rhs) = (as_copy(lhs), as_copy(rhs));
                    let local = |body: &mut Body| {
                        body.locals.push(ember_mir::LocalDecl { ty, kind: ember_mir::LocalKind::Temp, name: None, span });
                        Place::local(LocalId(body.locals.len() as u32 - 1))
                    };
                    let (r, s) = (local(body), local(body));
                    let set = |place: &Place, rvalue: Rvalue| Stmt::new(StmtKind::Assign { place: place.clone(), rvalue }, span);
                    let mut stmts = vec![
                        set(&r, binary(BinOp::Rem, lhs.clone(), rhs.clone())),
                        set(&s, binary(BinOp::Shr, Operand::Copy(r.clone()), int(width - 1))),
                    ];
                    if op == BinOp::FloorRem {
                        let m = local(body);
                        stmts.push(set(&m, binary(BinOp::BitAnd, Operand::Copy(s), rhs)));
                        stmts.push(assign(binary(BinOp::Add, Operand::Copy(r), Operand::Copy(m))));
                    } else {
                        let q = local(body);
                        stmts.push(set(&q, binary(BinOp::Div, lhs, rhs)));
                        stmts.push(assign(binary(BinOp::Add, Operand::Copy(q), Operand::Copy(s))));
                    }
                    stmts
                }
                _ => vec![assign(binary(op, lhs, rhs))],
            }
        }
        _ => vec![assign(binary(op, lhs, rhs))],
    };
    body.blocks[block].stmts.extend(stmts);
}

fn width_of(types: &TypeTable, ty: Ty) -> u32 {
    ember_types::bit_width(types, ty).unwrap_or(64) as u32
}

/// Whether unsigned division by the constant `d` at `width` bits is a
/// multiply by a magic number that fits the width, and a shift (Hacker's
/// Delight, 10-10): the smallest `p >= width` with
/// `2^p > nc * (d - 1 - (2^p - 1) mod d)` gives `M = (2^p + d - 1 - (2^p - 1) mod d) / d`;
/// when `M` needs `width + 1` bits the compiler adds a correction sequence.
fn unsigned_magic_fits(d: u128, width: u32) -> bool {
    if d < 2 || width == 0 || width > 64 || d >= (1u128 << width) {
        return false;
    }
    let two_n = 1u128 << width;
    let nc = two_n - 1 - (two_n - d) % d;
    for p in width..(2 * width).min(127) {
        let two_p = 1u128 << p;
        let r = (two_p - 1) % d;
        let Some(product) = nc.checked_mul(d - 1 - r) else { continue };
        if two_p > product {
            let magic = (two_p + d - 1 - r) / d;
            return magic < two_n;
        }
    }
    false
}

/// The unsigned integer type of `ty`'s width.
fn unsigned_of(types: &TypeTable, common: &CommonTypes, ty: Ty) -> Ty {
    match types.kind(ty) {
        TyKind::Int(ember_types::IntTy::I8) => common.u8,
        TyKind::Int(ember_types::IntTy::I16) => common.u16,
        TyKind::Int(ember_types::IntTy::I32) => common.u32,
        TyKind::Int(ember_types::IntTy::I64) => common.u64,
        TyKind::Int(ember_types::IntTy::I128) => common.u128,
        TyKind::Int(_) => common.usize,
        _ => ty,
    }
}

/// With its check gone, the comparison that fed it goes too when nothing else
/// reads it, so the C has no variable set and never read.
fn drop_unread_condition(body: &mut Body, block: usize, cond: LocalId) {
    let mut reads = 0;
    for data in &body.blocks {
        for stmt in &data.stmts {
            let mut count = |place: &Place| reads += usize::from(place.local == cond);
            match &stmt.kind {
                StmtKind::Assign { rvalue, .. } => {
                    for operand in rvalue_operands(rvalue) {
                        if let Operand::Copy(place) | Operand::Move(place) = operand {
                            count(place);
                        }
                    }
                    if let Rvalue::Ref { place, .. } | Rvalue::Discriminant(place) = rvalue {
                        count(place);
                    }
                }
                StmtKind::CheckedBinaryOp { lhs, rhs, .. } => {
                    for operand in [lhs, rhs] {
                        if let Operand::Copy(place) | Operand::Move(place) = operand {
                            count(place);
                        }
                    }
                }
                _ => {}
            }
        }
        let mut places = Vec::new();
        match &data.terminator {
            Terminator::SwitchInt { discr, .. } => places.push(discr),
            Terminator::Call { args, .. } => places.extend(args.iter()),
            Terminator::Assert { cond, msg, .. } => {
                places.push(cond);
                if let AssertKind::Bounds { len, index } = msg {
                    places.push(len);
                    places.push(index);
                }
            }
            _ => {}
        }
        for operand in places {
            if let Operand::Copy(place) | Operand::Move(place) = operand {
                reads += usize::from(place.local == cond);
            }
        }
    }
    if reads > 0 {
        return;
    }
    for stmt in body.blocks[block].stmts.iter_mut().rev() {
        if matches!(&stmt.kind, StmtKind::Assign { place, .. } if place.local == cond && place.projection.is_empty()) {
            stmt.kind = StmtKind::Nop;
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::unsigned_magic_fits;

    /// The divisors whose unsigned magic numbers are known: 3, 5, 13 and
    /// 10 fit 64 bits; 7 does not, at 64 or at 32 bits.
    #[test]
    fn unsigned_magic_numbers_match_the_known_ones() {
        for d in [3u128, 5, 10, 13] {
            assert!(unsigned_magic_fits(d, 64), "{d}");
        }
        assert!(!unsigned_magic_fits(7, 64));
        assert!(!unsigned_magic_fits(7, 32));
        assert!(unsigned_magic_fits(3, 32));
    }
}
