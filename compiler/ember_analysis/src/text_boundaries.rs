//! D-759, `[TXT-4]` — character boundaries and cursor order known by construction.
//!
//! A text slice `s[a..b]` checks `a <= b`, `b <= s.len()` and each bound for a character boundary
//! (`StrIsCharBoundary`, then an `Assert`), because a byte offset past the end or in the middle of a
//! character would cut a UTF-8 sequence. A text loop (`split_whitespace`, `split`, a hand-written
//! scanner) slices at cursors none of which can fail them: `0`, the text's length, and a cursor
//! `next_char` advanced from one of those, which steps over whole characters and so never passes
//! the end; and its word ends where it began or further on. `t5_words` ran 1.13x its C twin on
//! these checks, three bounds and four boundaries a word.
//!
//! What is known, per local: the texts it is a boundary of (so at most their length), the text
//! whose length it is, and which locals it is known to be at least. A text is named by a local
//! assigned once (a copy of such a local names the same text), so no fact outlives its text. A
//! cursor is followed only while every address taken of it goes to `next_char`; anything else that
//! may write a local ends what is known of it. A check these facts decide goes, and its branch.

use std::collections::{BTreeSet, HashMap, HashSet};

use ember_mir::{
    AssertKind, BinOp, Body, Builtin, CheckKind, CheckProof, Const, FuncRef, LocalId, Operand, Place, RemovedCheck, Rvalue,
    StmtKind, Terminator,
};
use ember_types::{IntTy, TyKind, TypeTable, UintTy};

/// Remove every text-slice check the facts decide, in each body; returns how many went.
pub fn remove_known_boundary_checks_all(bodies: &mut [Body], types: &TypeTable) -> usize {
    bodies.iter_mut().map(|body| remove_in_body(body, types)).sum()
}

/// Where a local is a character boundary: of every text (`0`), or of these texts.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Boundary {
    Any,
    Of(BTreeSet<LocalId>),
}

impl Boundary {
    fn meet(&self, other: &Boundary) -> Option<Boundary> {
        let met = match (self, other) {
            (Boundary::Any, fact) | (fact, Boundary::Any) => fact.clone(),
            (Boundary::Of(a), Boundary::Of(b)) => Boundary::Of(a.intersection(b).copied().collect()),
        };
        match &met {
            Boundary::Of(set) if set.is_empty() => None,
            _ => Some(met),
        }
    }

    fn holds_for(&self, text: LocalId) -> bool {
        match self {
            Boundary::Any => true,
            Boundary::Of(set) => set.contains(&text),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct State {
    boundary: HashMap<LocalId, Boundary>,
    /// `x -> t`: `x` holds the length of text `t`.
    length_of: HashMap<LocalId, LocalId>,
    /// `(a, b)`: `a <= b`, both non-negative.
    at_most: BTreeSet<(LocalId, LocalId)>,
}

impl State {
    fn meet(&self, other: &State) -> State {
        State {
            boundary: self
                .boundary
                .iter()
                .filter_map(|(local, fact)| other.boundary.get(local).and_then(|that| fact.meet(that)).map(|fact| (*local, fact)))
                .collect(),
            length_of: self.length_of.iter().filter(|(local, text)| other.length_of.get(local) == Some(text)).map(|(l, t)| (*l, *t)).collect(),
            at_most: self.at_most.intersection(&other.at_most).copied().collect(),
        }
    }

    fn kill(&mut self, local: LocalId) {
        self.boundary.remove(&local);
        self.length_of.remove(&local);
        self.at_most.retain(|(a, b)| *a != local && *b != local);
    }

    /// Whether `local` is known not to be negative: a boundary or a length.
    fn non_negative(&self, local: LocalId) -> bool {
        self.boundary.contains_key(&local) || self.length_of.contains_key(&local)
    }

    /// `dest` takes `source`'s value: every fact of it, and the two are equal.
    fn copy(&mut self, dest: LocalId, source: LocalId) {
        self.kill(dest);
        if let Some(fact) = self.boundary.get(&source).cloned() {
            self.boundary.insert(dest, fact);
        }
        if let Some(text) = self.length_of.get(&source).copied() {
            self.length_of.insert(dest, text);
        }
        if !self.non_negative(source) {
            return;
        }
        let pairs: Vec<(LocalId, LocalId)> = self.at_most.iter().copied().collect();
        for (a, b) in pairs {
            if b == source {
                self.at_most.insert((a, dest));
            }
            if a == source {
                self.at_most.insert((dest, b));
            }
        }
        self.at_most.insert((source, dest));
        self.at_most.insert((dest, source));
    }
}

struct Shape {
    /// Each local assigned once (arguments count as assigned on entry).
    assigned_once: HashSet<LocalId>,
    /// A once-assigned local that is a copy of another.
    copy_of: HashMap<LocalId, LocalId>,
    /// `t = &mut x` temporaries whose only use is a `next_char` cursor: `t -> x`.
    cursors: HashMap<LocalId, LocalId>,
    /// Locals whose address goes anywhere but a `next_char` cursor: nothing is known of them.
    escaped: HashSet<LocalId>,
}

fn plain(place: &Place) -> Option<LocalId> {
    place.projection.is_empty().then_some(place.local)
}

fn operand_local(operand: &Operand) -> Option<LocalId> {
    match operand {
        Operand::Copy(place) | Operand::Move(place) => plain(place),
        Operand::Const(_) => None,
    }
}

fn is_text_step(func: &FuncRef) -> bool {
    matches!(func, FuncRef::Builtin { which: Builtin::StrCharNext | Builtin::StrCharNextWide, .. })
}

fn shape(body: &Body) -> Shape {
    let mut writes: HashMap<LocalId, usize> = HashMap::new();
    for index in 1..=body.arg_count {
        *writes.entry(LocalId(index as u32)).or_default() += 1;
    }
    let mut copy_of = HashMap::new();
    let mut refs: HashMap<LocalId, LocalId> = HashMap::new();
    let mut escaped = HashSet::new();
    // Per temporary: its uses, and how many of them are a `next_char` cursor.
    let mut uses: HashMap<LocalId, (usize, usize)> = HashMap::new();
    let note = |operand: &Operand, as_cursor: bool, uses: &mut HashMap<LocalId, (usize, usize)>| {
        if let Some(local) = operand_local(operand) {
            let entry = uses.entry(local).or_default();
            entry.0 += 1;
            if as_cursor {
                entry.1 += 1;
            }
        }
    };
    for block in &body.blocks {
        for stmt in &block.stmts {
            match &stmt.kind {
                StmtKind::Assign { place, rvalue } => {
                    *writes.entry(place.local).or_default() += 1;
                    match rvalue {
                        Rvalue::Use(operand) => {
                            if let (Some(dest), Some(source)) = (plain(place), operand_local(operand)) {
                                copy_of.insert(dest, source);
                            }
                            note(operand, false, &mut uses);
                        }
                        Rvalue::Ref { place: target, mutable } => match (plain(place), *mutable) {
                            (Some(temp), true) if target.projection.is_empty() => {
                                refs.insert(temp, target.local);
                            }
                            _ => {
                                escaped.insert(target.local);
                            }
                        },
                        Rvalue::BinaryOp { lhs, rhs, .. } => {
                            note(lhs, false, &mut uses);
                            note(rhs, false, &mut uses);
                        }
                        Rvalue::UnaryOp { operand, .. } | Rvalue::Cast { operand, .. } => note(operand, false, &mut uses),
                        Rvalue::Aggregate { operands, .. } => {
                            for operand in operands {
                                note(operand, false, &mut uses);
                            }
                        }
                        Rvalue::Repeat { value, .. } => note(value, false, &mut uses),
                        Rvalue::Discriminant(_) => {}
                    }
                }
                StmtKind::CheckedBinaryOp { dest, overflow, lhs, rhs, .. } => {
                    *writes.entry(dest.local).or_default() += 1;
                    *writes.entry(overflow.local).or_default() += 1;
                    note(lhs, false, &mut uses);
                    note(rhs, false, &mut uses);
                }
                StmtKind::Drop { place, .. } => {
                    *writes.entry(place.local).or_default() += 1;
                }
                _ => {}
            }
        }
        match &block.terminator {
            Terminator::Call { func, args, dest, .. } => {
                *writes.entry(dest.local).or_default() += 1;
                let step = is_text_step(func);
                for (index, arg) in args.iter().enumerate() {
                    note(arg, step && index == 1, &mut uses);
                }
            }
            Terminator::SwitchInt { discr, .. } | Terminator::Assert { cond: discr, .. } => note(discr, false, &mut uses),
            _ => {}
        }
    }
    let assigned_once: HashSet<LocalId> = writes.iter().filter(|(_, count)| **count == 1).map(|(local, _)| *local).collect();
    copy_of.retain(|dest, _| assigned_once.contains(dest));
    let mut cursors = HashMap::new();
    for (temp, target) in refs {
        let (all, as_cursor) = uses.get(&temp).copied().unwrap_or((0, 0));
        if assigned_once.contains(&temp) && all == as_cursor {
            cursors.insert(temp, target);
        } else {
            escaped.insert(target);
        }
    }
    Shape { assigned_once, copy_of, cursors, escaped }
}

impl Shape {
    /// The text a once-assigned local names, following copies; `None` for a local assigned more
    /// than once, whose value a fact could outlive.
    fn text(&self, mut local: LocalId) -> Option<LocalId> {
        if !self.assigned_once.contains(&local) {
            return None;
        }
        for _ in 0..64 {
            match self.copy_of.get(&local) {
                Some(&source) if self.assigned_once.contains(&source) => local = source,
                _ => break,
            }
        }
        Some(local)
    }
}

/// A 64-bit integer type: a cursor, a length, and every copy of them through `int` and `usize`.
fn wide_integer(types: &TypeTable, ty: ember_types::Ty) -> bool {
    matches!(types.kind(ty), TyKind::Int(IntTy::I64 | IntTy::Isize) | TyKind::Uint(UintTy::U64 | UintTy::Usize))
}

fn tracked(body: &Body, types: &TypeTable, shape: &Shape, local: LocalId) -> bool {
    !shape.escaped.contains(&local) && wide_integer(types, body.locals[local.0 as usize].ty)
}

fn boundary_of(state: &State, operand: &Operand) -> Option<Boundary> {
    match operand {
        Operand::Const(Const::Int { value: 0, .. }) => Some(Boundary::Any),
        Operand::Copy(place) | Operand::Move(place) => plain(place).and_then(|local| state.boundary.get(&local).cloned()),
        Operand::Const(_) => None,
    }
}

fn transfer_stmt(state: &mut State, stmt: &ember_mir::Stmt, body: &Body, types: &TypeTable, shape: &Shape) {
    match &stmt.kind {
        StmtKind::Assign { place, rvalue } => {
            let Some(dest) = plain(place) else {
                state.kill(place.local);
                return;
            };
            if !tracked(body, types, shape, dest) {
                state.kill(dest);
                return;
            }
            match rvalue {
                Rvalue::Use(Operand::Const(Const::Int { value: 0, .. })) => {
                    state.kill(dest);
                    state.boundary.insert(dest, Boundary::Any);
                }
                Rvalue::Use(operand) => match operand_local(operand) {
                    Some(source) => state.copy(dest, source),
                    None => state.kill(dest),
                },
                // Between 64-bit integers, a cast keeps the value of a length or a cursor (each at
                // least 0 and below 2^63), and so every fact of it.
                Rvalue::Cast { operand, to, .. } if wide_integer(types, *to) => match operand_local(operand) {
                    Some(source) if state.non_negative(source) => state.copy(dest, source),
                    _ => state.kill(dest),
                },
                _ => state.kill(dest),
            }
        }
        StmtKind::CheckedBinaryOp { dest, overflow, .. } => {
            state.kill(dest.local);
            state.kill(overflow.local);
        }
        StmtKind::Drop { place, .. } => state.kill(place.local),
        StmtKind::StorageLive(local) | StmtKind::StorageDead(local) => state.kill(*local),
        _ => {}
    }
}

fn transfer_terminator(state: &mut State, terminator: &Terminator, body: &Body, types: &TypeTable, shape: &Shape) {
    let Terminator::Call { func, args, dest, .. } = terminator else { return };
    match func {
        // A text's length is a boundary of it.
        FuncRef::Builtin { which: Builtin::SpanLen, arg_ty } if matches!(types.kind(*arg_ty), TyKind::Str) => {
            state.kill(dest.local);
            let text = args.first().and_then(operand_local).and_then(|local| shape.text(local));
            if let (Some(dest), Some(text)) = (plain(dest), text)
                && tracked(body, types, shape, dest)
            {
                state.boundary.insert(dest, Boundary::Of(BTreeSet::from([text])));
                state.length_of.insert(dest, text);
            }
        }
        // One whole character on from a boundary is a boundary, and no less than the cursor was.
        func if is_text_step(func) => {
            state.kill(dest.local);
            let text = args.first().and_then(operand_local).and_then(|local| shape.text(local));
            let Some(cursor) = args.get(1).and_then(operand_local).and_then(|temp| shape.cursors.get(&temp).copied()) else {
                return;
            };
            let stays = matches!((text, state.boundary.get(&cursor)), (Some(text), Some(fact)) if fact.holds_for(text));
            if !stays {
                state.kill(cursor);
                return;
            }
            let text = text.expect("checked above");
            state.boundary.insert(cursor, Boundary::Of(BTreeSet::from([text])));
            state.length_of.remove(&cursor);
            // It grew: what it was at least, it still is; what it was at most, it may not be.
            state.at_most.retain(|(a, _)| *a != cursor);
        }
        _ => state.kill(dest.local),
    }
}

/// Whether `lhs <= rhs` holds in `state`.
fn known_at_most(state: &State, lhs: &Operand, rhs: &Operand) -> bool {
    match (lhs, operand_local(lhs), operand_local(rhs)) {
        (Operand::Const(Const::Int { value: 0, .. }), _, Some(r)) => state.non_negative(r),
        (_, Some(l), Some(r)) => {
            l == r && state.non_negative(l)
                || state.at_most.contains(&(l, r))
                || state.length_of.get(&r).is_some_and(|text| state.boundary.get(&l).is_some_and(|fact| fact.holds_for(*text)))
        }
        _ => false,
    }
}

fn remove_in_body(body: &mut Body, types: &TypeTable) -> usize {
    let has_text = body.blocks.iter().any(|block| {
        matches!(&block.terminator, Terminator::Call { func: FuncRef::Builtin { which: Builtin::StrIsCharBoundary, .. }, .. })
    });
    if !has_text {
        return 0;
    }
    let shape = shape(body);
    // Forward, to a fixed point: a block's entry facts are the meet of its visited predecessors'.
    let count = body.blocks.len();
    let mut entry: Vec<Option<State>> = vec![None; count];
    entry[0] = Some(State::default());
    let mut exits: Vec<Option<State>> = vec![None; count];
    let mut work: Vec<usize> = vec![0];
    let mut rounds = 0usize;
    while let Some(block) = work.pop() {
        rounds += 1;
        if rounds > count * 64 + 1024 {
            return 0; // no fixed point in reach: remove nothing
        }
        let mut state = entry[block].clone().unwrap_or_default();
        for stmt in &body.blocks[block].stmts {
            transfer_stmt(&mut state, stmt, body, types, &shape);
        }
        transfer_terminator(&mut state, &body.blocks[block].terminator, body, types, &shape);
        if exits[block].as_ref() == Some(&state) {
            continue;
        }
        exits[block] = Some(state.clone());
        for next in successors(&body.blocks[block].terminator) {
            let merged = match &entry[next] {
                None => state.clone(),
                Some(old) => old.meet(&state),
            };
            if entry[next].as_ref() != Some(&merged) || exits[next].is_none() {
                entry[next] = Some(merged);
                work.push(next);
            }
        }
    }

    // The checks the facts decide: a boundary check (`ok = StrIsCharBoundary(text, bound)` ending a
    // block whose next block asserts `ok`), and a bounds check (`ok = lhs <= rhs` ending a block
    // that asserts `ok`).
    let mut boundary_checks = Vec::new();
    let mut bounds_checks = Vec::new();
    for (index, block) in body.blocks.iter().enumerate() {
        let Some(mut state) = entry[index].clone() else { continue };
        let last = block.stmts.len();
        for (position, stmt) in block.stmts.iter().enumerate() {
            if position + 1 == last
                && let StmtKind::Assign { place, rvalue: Rvalue::BinaryOp { op: BinOp::Le, lhs, rhs } } = &stmt.kind
                && let Terminator::Assert { cond: Operand::Copy(cond), expected: true, msg: AssertKind::Bounds { .. }, .. } = &block.terminator
                && cond == place
                && known_at_most(&state, lhs, rhs)
            {
                bounds_checks.push(index);
            }
            transfer_stmt(&mut state, stmt, body, types, &shape);
        }
        let Terminator::Call { func: FuncRef::Builtin { which: Builtin::StrIsCharBoundary, .. }, args, dest, next } = &block.terminator else {
            continue;
        };
        let Some(text) = args.first().and_then(operand_local).and_then(|local| shape.text(local)) else { continue };
        let known = args.get(1).and_then(|bound| boundary_of(&state, bound)).is_some_and(|fact| fact.holds_for(text));
        let assert_block = next.0 as usize;
        let asserts_dest = body.blocks[assert_block].stmts.is_empty()
            && matches!(
                &body.blocks[assert_block].terminator,
                Terminator::Assert { cond: Operand::Copy(cond), expected: true, .. } if cond == dest
            );
        if known && asserts_dest {
            boundary_checks.push((index, assert_block, dest.clone()));
        }
    }
    for &block in &bounds_checks {
        let span = body.blocks[block].terminator_span;
        let Terminator::Assert { next, .. } = body.blocks[block].terminator.clone() else { unreachable!("checked above") };
        body.blocks[block].terminator = Terminator::Goto(next);
        body.removed_checks.push(RemovedCheck { span, kind: CheckKind::Bounds, proof: CheckProof::RangeFacts });
    }
    for (check, assert_block, dest) in &boundary_checks {
        let span = body.blocks[*check].terminator_span;
        body.blocks[*check].stmts.push(ember_mir::Stmt {
            kind: StmtKind::Assign { place: dest.clone(), rvalue: Rvalue::Use(Operand::Const(Const::Bool(true))) },
            span,
        });
        body.blocks[*check].terminator = Terminator::Goto(ember_mir::BasicBlockId(*assert_block as u32));
        let Terminator::Assert { next, .. } = body.blocks[*assert_block].terminator.clone() else { unreachable!("checked above") };
        body.blocks[*assert_block].terminator = Terminator::Goto(next);
    }
    bounds_checks.len() + boundary_checks.len()
}

fn successors(terminator: &Terminator) -> Vec<usize> {
    match terminator {
        Terminator::Goto(target) => vec![target.0 as usize],
        Terminator::SwitchInt { targets, otherwise, .. } => {
            targets.iter().map(|(_, target)| target.0 as usize).chain(std::iter::once(otherwise.0 as usize)).collect()
        }
        Terminator::Call { next, .. } | Terminator::Assert { next, .. } => vec![next.0 as usize],
        Terminator::Return | Terminator::Unreachable => Vec::new(),
    }
}
