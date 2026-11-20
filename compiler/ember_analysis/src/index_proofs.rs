//! `[BRW-5]`, `[BRW-1]`, `[BCK-3]` — two element places of one list whose indexes the code proves
//! different are two places: "both constants and different, `j` from `i + 1..`, or the line inside
//! `if i != j:`". Nothing is compared while the program runs; each proof is read from the body.
//!
//! An index is followed back through the copies and conversions MIR makes of it (`_15 = copy _14`,
//! `_14 = Numeric(copy _1) as usize`) to what it stands for: a constant, or a local that no line
//! changes while the proof is used. A proof holds:
//! - for two different constants, everywhere;
//! - for a counter `j` whose every write is `j = i + k` or `j = j + k` with `k >= 1`, where `i` is
//!   written at most once and before it (a parameter, or a local set once): `j > i` wherever `j` has a
//!   value, so `j` and `i` differ (a `for j in i + 1..n:` loop lowers to exactly this);
//! - for `i` and `j` compared `!=` (or `==`, on its false arm) by a branch, in every block the arm
//!   taken dominates and only that branch enters, when neither is written after the comparison (each
//!   written at most once, a parameter or set once).

use std::collections::{HashMap, HashSet};

use ember_mir::{BinOp, Body, Const, LocalId, LocalKind, Operand, Place, Projection, Rvalue, StmtKind, Terminator};
use ember_types::{IntTy, TyKind, TypeTable, UintTy};

use crate::loop_version::{dominators, visit_places};

/// What an index stands for, once its copies and conversions are followed.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Value {
    Constant(i128),
    Local(LocalId),
}

pub(crate) struct IndexProofs {
    /// How many times each local is written (assigned, a call's destination, borrowed `mut`).
    writes: HashMap<LocalId, usize>,
    /// The one value a local written once was given, when it is a copy, a conversion or a constant.
    copied: HashMap<LocalId, Operand>,
    /// Pairs of locals proved different, in a set of blocks (`None`: in every block).
    apart: Vec<(LocalId, LocalId, Option<HashSet<usize>>)>,
}

impl IndexProofs {
    pub(crate) fn new(body: &Body, types: &TypeTable) -> Self {
        let mut writes: HashMap<LocalId, usize> = HashMap::new();
        for block in &body.blocks {
            visit_places(block, &mut |place: &Place, write: bool, _| {
                if write {
                    *writes.entry(place.local).or_default() += 1;
                }
            });
        }
        let mut copied = HashMap::new();
        for block in &body.blocks {
            for stmt in &block.stmts {
                // Only the compiler's own temporaries are followed: a name the program wrote stands
                // for itself, so `i = 0` and `j = 1` are two locals and not two constants (the list of
                // proofs is closed, `decisions.md` §4, area 2 Q2), while `bodies[FIRST]` reads `FIRST`
                // through a temporary.
                if let StmtKind::Assign { place, rvalue } = &stmt.kind
                    && place.projection.is_empty()
                    && writes.get(&place.local) == Some(&1)
                    && body.local(place.local).kind == LocalKind::Temp
                {
                    match rvalue {
                        Rvalue::Use(operand) => {
                            copied.insert(place.local, operand.clone());
                        }
                        // A conversion keeps two different numbers different only when it loses no bits:
                        // from a number of at most 64 bits to a 64-bit index.
                        Rvalue::Cast { operand, .. } if operand_bits(body, types, operand).is_some_and(|bits| bits <= 64) => {
                            copied.insert(place.local, operand.clone());
                        }
                        _ => {}
                    }
                }
            }
        }
        let mut proofs = IndexProofs { writes, copied, apart: Vec::new() };
        proofs.counters(body);
        proofs.branches(body);
        proofs
    }

    /// Whether the index places `a` and `b` (of one list) are proved to be different elements in
    /// the blocks `at`.
    pub(crate) fn apart(&self, a: &Projection, b: &Projection, at: &[usize]) -> bool {
        let (Some(x), Some(y)) = (self.index_value(a), self.index_value(b)) else { return false };
        match (x, y) {
            (Value::Constant(x), Value::Constant(y)) => x != y,
            (Value::Local(x), Value::Local(y)) if x != y => self.apart.iter().any(|(p, q, blocks)| {
                ((*p, *q) == (x, y) || (*p, *q) == (y, x))
                    && blocks.as_ref().is_none_or(|blocks| at.iter().all(|block| blocks.contains(block)))
            }),
            _ => false,
        }
    }

    fn index_value(&self, projection: &Projection) -> Option<Value> {
        match projection {
            Projection::ConstIndex(index) => Some(Value::Constant(*index as i128)),
            Projection::Index(local) => Some(self.value_of(&Operand::Copy(Place::local(*local)))),
            _ => None,
        }
    }

    /// What `operand` stands for, through copies and conversions of locals written once.
    fn value_of(&self, operand: &Operand) -> Value {
        let mut operand = operand.clone();
        for _ in 0..16 {
            match &operand {
                Operand::Const(Const::Int { value, .. }) => return Value::Constant(*value as i128),
                Operand::Copy(place) | Operand::Move(place) if place.projection.is_empty() => {
                    match self.copied.get(&place.local) {
                        Some(next) => operand = next.clone(),
                        None => return Value::Local(place.local),
                    }
                }
                _ => break,
            }
        }
        // A value this cannot follow stands only for itself: a fresh local proved nothing about.
        Value::Local(LocalId(u32::MAX))
    }

    /// A local no line changes once it has its value: a parameter never written, or one written once.
    fn stable(&self, body: &Body, local: LocalId) -> bool {
        let written = self.writes.get(&local).copied().unwrap_or(0);
        let parameter = local.0 >= 1 && (local.0 as usize) <= body.arg_count;
        if parameter { written == 0 } else { written == 1 }
    }

    /// `j` from `i + 1..`: every write of `j` adds a positive constant to a stable `i` or to `j`.
    fn counters(&mut self, body: &Body) {
        let mut adds: HashMap<LocalId, Vec<(Operand, Operand, BinOp)>> = HashMap::new();
        for block in &body.blocks {
            for stmt in &block.stmts {
                match &stmt.kind {
                    StmtKind::Assign { place, rvalue: Rvalue::BinaryOp { op, lhs, rhs } } if place.projection.is_empty() => {
                        adds.entry(place.local).or_default().push((lhs.clone(), rhs.clone(), *op));
                    }
                    StmtKind::CheckedBinaryOp { dest, op, lhs, rhs, .. } if dest.projection.is_empty() => {
                        adds.entry(dest.local).or_default().push((lhs.clone(), rhs.clone(), *op));
                    }
                    _ => {}
                }
            }
        }
        for (counter, writes) in adds {
            // Every write of the counter is one of these adds: nothing else (a call, a `ref mut`) sets it.
            if self.writes.get(&counter).copied().unwrap_or(0) != writes.len() {
                continue;
            }
            let positive = |operand: &Operand| matches!(self.value_of(operand), Value::Constant(k) if k >= 1);
            let mut base = None;
            let mut ok = true;
            for (lhs, rhs, op) in &writes {
                if *op != BinOp::Add || !positive(rhs) {
                    ok = false;
                    break;
                }
                match self.value_of(lhs) {
                    Value::Local(local) if local == counter => {}
                    Value::Local(local) if self.stable(body, local) && base.is_none_or(|b| b == local) => base = Some(local),
                    _ => {
                        ok = false;
                        break;
                    }
                }
            }
            if ok && let Some(base) = base {
                self.apart.push((counter, base, None));
            }
        }
    }

    /// The line inside `if i != j:`: the arm a comparison of two stable locals took.
    fn branches(&mut self, body: &Body) {
        let all: Vec<usize> = (0..body.blocks.len()).collect();
        let mut dominated: Option<HashMap<usize, HashSet<usize>>> = None;
        let mut predecessors: HashMap<usize, usize> = HashMap::new();
        for block in &body.blocks {
            for next in successors(&block.terminator) {
                *predecessors.entry(next).or_default() += 1;
            }
        }
        for block in &body.blocks {
            let Terminator::SwitchInt { discr: Operand::Copy(test) | Operand::Move(test), targets, otherwise } = &block.terminator else {
                continue;
            };
            if !test.projection.is_empty() || targets.len() != 1 || targets[0].0 != 0 {
                continue;
            }
            let Some(compare) = body.blocks.iter().flat_map(|b| &b.stmts).find_map(|stmt| match &stmt.kind {
                StmtKind::Assign { place, rvalue: Rvalue::BinaryOp { op, lhs, rhs } }
                    if place.local == test.local && place.projection.is_empty() && self.writes.get(&test.local) == Some(&1) =>
                {
                    Some((*op, lhs.clone(), rhs.clone()))
                }
                _ => None,
            }) else {
                continue;
            };
            // The arm on which the two differ: `!=` true (the otherwise arm), `==` false (the 0 arm).
            let arm = match compare.0 {
                BinOp::Ne => otherwise.0 as usize,
                BinOp::Eq => targets[0].1.0 as usize,
                _ => continue,
            };
            let (Value::Local(x), Value::Local(y)) = (self.value_of(&compare.1), self.value_of(&compare.2)) else { continue };
            if x == y || !self.stable(body, x) || !self.stable(body, y) || predecessors.get(&arm) != Some(&1) {
                continue;
            }
            let dominated = dominated.get_or_insert_with(|| dominators(body, 0, &all));
            let inside: HashSet<usize> =
                dominated.iter().filter(|(_, by)| by.contains(&arm)).map(|(&block, _)| block).collect();
            self.apart.push((x, y, Some(inside)));
        }
    }
}

fn successors(terminator: &Terminator) -> Vec<usize> {
    crate::long_access_lint::successors(terminator)
}

/// The width of a whole-number operand's type.
fn operand_bits(body: &Body, types: &TypeTable, operand: &Operand) -> Option<u32> {
    let ty = match operand {
        Operand::Copy(place) | Operand::Move(place) if place.projection.is_empty() => body.local(place.local).ty,
        Operand::Const(Const::Int { ty, .. }) => *ty,
        _ => return None,
    };
    match types.kind(ty) {
        TyKind::Int(IntTy::I8) | TyKind::Uint(UintTy::U8) => Some(8),
        TyKind::Int(IntTy::I16) | TyKind::Uint(UintTy::U16) => Some(16),
        TyKind::Int(IntTy::I32) | TyKind::Uint(UintTy::U32) => Some(32),
        TyKind::Int(IntTy::I64 | IntTy::Isize) | TyKind::Uint(UintTy::U64 | UintTy::Usize) => Some(64),
        _ => None,
    }
}
