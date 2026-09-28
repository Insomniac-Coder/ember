//! `[RNG-4]` — range facts over the MIR, and the checks they remove.
//!
//! For every whole-number local, and the length of every list or span a
//! function reads, the analysis keeps the interval of values it can hold at
//! each point, and facts of the form `a <= b + c` between two of them. Facts
//! come from constants, a value's type (a range type's declared range,
//! `[RNG-9]`), arithmetic on known ranges, copies and conversions, the arm of
//! a comparison a branch took, a check that passed, and a list's `len()`.
//! Loops are handled by widening at their headers and then narrowing once, so
//! a `for i in a..b` counter carries `a <= i < b` in its body.
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
//! A local whose address is taken mutably is not tracked, since a write
//! through the reference would be invisible. A list's length is forgotten
//! at anything that can change the list: a write to it or a place holding it,
//! a move, a mutable borrow, and — when some mutable borrow of it exists
//! anywhere in the function — every call. A list reached through a shared
//! reference cannot change while the reference lives (`[UNS-4]`).

use std::collections::{BTreeMap, HashMap, HashSet};

use ember_mir::{
    AssertKind, BinOp, Body, Builtin, CastKind, CheckKind, CheckProof, Const, FuncRef, LocalId, Operand, Place,
    Projection, RemovedCheck, Rvalue, Stmt, StmtKind, Terminator, UnOp,
};
use ember_types::{Bound, CommonTypes, Ty, TyKind, TypeTable};

use crate::regions::place_type;

/// Remove every check the range facts prove cannot fail; returns how many.
pub fn remove_proven_checks_all(bodies: &mut [Body], types: &TypeTable, common: &CommonTypes) -> usize {
    bodies.iter_mut().map(|body| remove_proven_checks(body, types, common)).sum()
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

/// The values a type holds: an integer's whole range, or a range type's
/// declared one (`[RNG-9]`). `None` for anything else, and for `u128`, whose
/// top half no `i128` holds.
pub(crate) fn type_range(types: &TypeTable, ty: Ty) -> Option<Interval> {
    match types.kind(ty) {
        TyKind::Int(_) | TyKind::Uint(_) => {
            let max = i128::try_from(ember_types::int_max(types, ty)?).ok()?;
            let lo = if ember_types::is_signed(types, ty) == Some(true) { -max - 1 } else { 0 };
            Some(Interval { lo, hi: max })
        }
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

/// The integer type a value of `ty` is represented by.
fn representation(types: &TypeTable, ty: Ty) -> Ty {
    match types.kind(ty) {
        TyKind::Range(id) => types.range_def(*id).repr,
        _ => ty,
    }
}

/// A constant's bits as a number of type `ty`.
fn constant(types: &TypeTable, value: u128, ty: Ty) -> Option<i128> {
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
    /// relation that loosened dropped: a loop header reaches a fixpoint.
    fn widen(&self, next: &State, analysis: &Analysis) -> State {
        let mut ranges = BTreeMap::new();
        for (var, old) in &self.ranges {
            let Some(new) = next.ranges.get(var) else { continue };
            let full = analysis.var_range(*var);
            let lo = if new.lo < old.lo { full.lo } else { old.lo };
            let hi = if new.hi > old.hi { full.hi } else { old.hi };
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

pub(crate) struct Analysis<'a> {
    body: &'a Body,
    types: &'a TypeTable,
    /// Per local: a whole integer local no reference can write.
    tracked: Vec<bool>,
    views: Vec<View>,
    usize_range: Interval,
    /// The state on entry to each block; `None` when unreachable.
    entry: Vec<Option<State>>,
}

/// Rounds before a body is left without facts: a guard, since widening
/// bounds every chain.
const MAX_ROUNDS: usize = 200;

impl<'a> Analysis<'a> {
    /// The facts of `body`, or `None` when they did not settle.
    pub(crate) fn run(body: &'a Body, types: &'a TypeTable, common: &CommonTypes) -> Option<Analysis<'a>> {
        let usize_range = type_range(types, common.usize)?;
        let mut analysis =
            Analysis { body, types, tracked: Vec::new(), views: Vec::new(), usize_range, entry: Vec::new() };
        analysis.tracked = analysis.tracked_locals();
        analysis.views = analysis.collect_views();
        analysis.solve()?;
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
            Var::Len(_) => self.usize_range,
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
                type_range(self.types, place_type(self.body, self.types, place))
            }
            _ => None,
        }
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
                for arg in args {
                    self.forget_moved(&mut state, arg);
                }
                for (index, view) in self.views.iter().enumerate() {
                    if view.lent {
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

    fn solve(&mut self) -> Option<()> {
        let n = self.body.blocks.len();
        let order = reverse_postorder(self.body);
        let headers = loop_headers(self.body, &order);
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
                                old.widen(&joined, self)
                            } else {
                                joined
                            }
                        }
                    };
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
                    next[target] = Some(match &next[target] {
                        None => incoming,
                        Some(old) => old.join(&incoming),
                    });
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

fn reverse_postorder(body: &Body) -> Vec<usize> {
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
fn loop_headers(body: &Body, order: &[usize]) -> HashSet<usize> {
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

// ---------------------------------------------------------------------------
// Removing the checks.

fn remove_proven_checks(body: &mut Body, types: &TypeTable, common: &CommonTypes) -> usize {
    // What to change, found on the unchanged body.
    struct Removal {
        block: usize,
        kind: CheckKind,
        /// The checked operation to make plain, with its operand ranges.
        checked: Option<(Interval, Interval)>,
    }
    let removals: Vec<Removal> = {
        let Some(analysis) = Analysis::run(body, types, common) else { return 0 };
        let mut removals = Vec::new();
        for block in 0..body.blocks.len() {
            let Some(entry) = analysis.entry[block].clone() else { continue };
            let data = &body.blocks[block];
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
    removals.len()
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
                _ if a.lo >= 0 && b.lo > 0 => {
                    let unsigned = unsigned_of(types, common, ty);
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
