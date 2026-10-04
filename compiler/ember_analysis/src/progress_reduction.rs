//! Late semantic-progress reduction with an independently verified guard.
//! One pure DAG epoch, one signed64 nonnegative ADD, private usize decoder.
//! No benchmark names, AST patterns, new builtin, or CountedLoop relaxation.
use crate::definite_init::{analyze_definite_init, verify_initialization_facts};
use crate::range_facts::{constant, loop_headers, natural_loops, reverse_postorder, type_range};
use ember_mir::{
    AssertKind, BasicBlock, BasicBlockId, BinOp, Body, Builtin, CastKind, CheckKind, CheckProof,
    Const, FuncRef, LocalDecl, LocalId, LocalKind, Operand, Place, Projection, RemovedCheck,
    Rvalue, Stmt, StmtKind, Terminator,
};
use ember_span::Span;
use ember_types::{CommonTypes, Ty, TypeTable};
use std::collections::{BTreeMap, BTreeSet};

const LIMIT: u64 = i64::MAX as u64;
const MAX_BLOCKS: usize = 4096;
const MAX_STMTS: usize = 65536;
const MAX_INIT_CELLS: usize = 262144; // whole-body block × local lattice cells

/// Driver-owned, private evidence. Not a trusted "safe" boolean/checksum.
/// No later mutating pass may run between verification and for_codegen.
struct ProgressReductionCertificate {
    body_index: usize,
    original: Body,
    header: usize,
}
#[must_use = "retain the complete proof batch for unconditional final verification"]
pub struct ProgressReductionCertificates {
    body_count: usize,
    planned_count: usize,
    entries: Vec<ProgressReductionCertificate>,
}
impl ProgressReductionCertificates {
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    pub fn count(&self) -> usize {
        self.entries.len()
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
enum Upper {
    Constant(u64),
    Length,
    BeforeStep,
    Sum(Box<Upper>, Box<Upper>, u64),
    Min(Box<Upper>, Box<Upper>),
}
#[derive(Clone, PartialEq, Eq, Debug)]
struct Plan {
    header: usize,
    region: BTreeSet<usize>,
    test: usize,
    step: usize,
    view: LocalId,
    cursor: LocalId,
    reference: LocalId,
    check: usize,
    statement: usize,
    acc: LocalId,
    flag: LocalId,
    upper: Upper,
}
#[derive(Clone, Copy)]
struct Point {
    block: usize,
    statement: usize,
}
type Defs = BTreeMap<LocalId, Vec<Point>>;

fn copy(local: LocalId) -> Operand {
    Operand::Copy(Place::local(local))
}
fn int(value: u64, ty: Ty) -> Operand {
    Operand::Const(Const::Int {
        value: value as u128,
        ty,
    })
}
fn local(op: &Operand) -> Option<LocalId> {
    if let Operand::Copy(p) = op {
        if p.projection.is_empty() {
            return Some(p.local);
        }
    }
    None
}
fn succ(t: &Terminator) -> Vec<usize> {
    match t {
        Terminator::Goto(b)
        | Terminator::Call { next: b, .. }
        | Terminator::Assert { next: b, .. } => vec![b.0 as usize],
        Terminator::SwitchInt {
            targets, otherwise, ..
        } => targets
            .iter()
            .map(|x| x.1.0 as usize)
            .chain([otherwise.0 as usize])
            .collect(),
        _ => Vec::new(),
    }
}
fn retarget(t: &mut Terminator, from: usize, to: usize) {
    crate::loop_version::retarget(t, |b| {
        if b.0 as usize == from {
            BasicBlockId(to as u32)
        } else {
            b
        }
    });
}
fn budget(body: &Body) -> bool {
    !body.blocks.is_empty()
        && body.blocks.len() <= MAX_BLOCKS
        && body.locals.len() <= 65536
        && body
            .blocks
            .len()
            .checked_mul(body.locals.len())
            .is_some_and(|cells| cells <= MAX_INIT_CELLS)
        && body.blocks.iter().map(|b| b.stmts.len()).sum::<usize>() <= MAX_STMTS
        && body
            .blocks
            .iter()
            .map(|b| match &b.terminator {
                Terminator::SwitchInt { targets, .. } => targets.len() + 1,
                Terminator::Return | Terminator::Unreachable => 0,
                _ => 1,
            })
            .sum::<usize>()
            <= MAX_STMTS
}
fn definitions(body: &Body, blocks: &BTreeSet<usize>) -> Defs {
    let mut out: Defs = BTreeMap::new();
    for &b in blocks {
        for (i, s) in body.blocks[b].stmts.iter().enumerate() {
            let locals: Vec<LocalId> = match &s.kind {
                StmtKind::Assign { place, .. } if place.projection.is_empty() => vec![place.local],
                StmtKind::CheckedBinaryOp { dest, overflow, .. } => {
                    vec![dest.local, overflow.local]
                }
                _ => Vec::new(),
            };
            for l in locals {
                out.entry(l).or_default().push(Point {
                    block: b,
                    statement: i,
                });
            }
        }
        if let Terminator::Call { dest, .. } = &body.blocks[b].terminator {
            out.entry(dest.local).or_default().push(Point {
                block: b,
                statement: body.blocks[b].stmts.len(),
            });
        }
    }
    out
}
fn unique(defs: &Defs, l: LocalId) -> Option<Point> {
    let [p] = defs.get(&l)?.as_slice() else {
        return None;
    };
    Some(*p)
}
fn dominators(
    body: &Body,
    region: &BTreeSet<usize>,
    header: usize,
) -> Option<BTreeMap<usize, BTreeSet<usize>>> {
    let mut dom: BTreeMap<_, _> = region
        .iter()
        .map(|&b| {
            (
                b,
                if b == header {
                    BTreeSet::from([header])
                } else {
                    region.clone()
                },
            )
        })
        .collect();
    for _ in 0..=region.len() {
        let mut changed = false;
        for &b in region {
            if b == header {
                continue;
            }
            let preds: Vec<_> = region
                .iter()
                .copied()
                .filter(|&p| succ(&body.blocks[p].terminator).contains(&b))
                .collect();
            if preds.is_empty() {
                return None;
            }
            let mut next = dom[&preds[0]].clone();
            for p in &preds[1..] {
                next = next.intersection(&dom[p]).copied().collect();
            }
            next.insert(b);
            if next != dom[&b] {
                dom.insert(b, next);
                changed = true;
            }
        }
        if !changed {
            return Some(dom);
        }
    }
    None
}
fn dominates(dom: &BTreeMap<usize, BTreeSet<usize>>, a: Point, b: Point) -> bool {
    if a.block == b.block {
        a.statement < b.statement
    } else {
        dom[&b.block].contains(&a.block)
    }
}
fn readable(body: &Body, block: usize, locals: &[LocalId]) -> bool {
    let facts = analyze_definite_init(body);
    verify_initialization_facts(body, &facts).is_empty()
        && facts.block_entry(block).is_some_and(|s| {
            locals
                .iter()
                .all(|l| s.get(l.0 as usize).is_some_and(|v| v.is_readable()))
        })
}

// All outside incoming paths must reach an exact zero definition. A cycle or
// entry without such a definition rejects; outer-loop initialization qualifies.
fn zero_entry(
    body: &Body,
    region: &BTreeSet<usize>,
    header: usize,
    cursor: LocalId,
    common: &CommonTypes,
) -> bool {
    fn walk(
        body: &Body,
        region: &BTreeSet<usize>,
        b: usize,
        c: LocalId,
        ty: Ty,
        predecessors: &[Vec<usize>],
        fuel: &mut usize,
        depth: usize,
        active: &mut BTreeSet<usize>,
        done: &mut BTreeSet<usize>,
    ) -> bool {
        if done.contains(&b) {
            return true;
        }
        if depth > 128 || *fuel == 0 {
            return false;
        }
        *fuel -= 1;
        if region.contains(&b) || !active.insert(b) {
            return false;
        }
        if term_moves(&body.blocks[b].terminator, c) {
            return false;
        }
        if matches!(&body.blocks[b].terminator, Terminator::Call { dest, .. } if dest.local == c) {
            return false;
        }
        for s in body.blocks[b].stmts.iter().rev() {
            if matches!(&s.kind,StmtKind::Assign{rvalue,..} if rv_moves(rvalue,c))
                || matches!(&s.kind,StmtKind::CheckedBinaryOp{lhs,rhs,..} if operand_moves(lhs,c)||operand_moves(rhs,c))
                || matches!(&s.kind,StmtKind::Drop{place,..} if place.local==c)
            {
                return false;
            }
            match &s.kind {
                StmtKind::Assign { place, rvalue } if place.local == c => {
                    let yes = place.projection.is_empty()
                        && matches!(rvalue,
                        Rvalue::Use(Operand::Const(Const::Int { value: 0, ty: t })) if *t == ty);
                    active.remove(&b);
                    if yes {
                        done.insert(b);
                    }
                    return yes;
                }
                StmtKind::CheckedBinaryOp { dest, overflow, .. }
                    if dest.local == c || overflow.local == c =>
                {
                    return false;
                }
                StmtKind::StorageLive(l) | StmtKind::StorageDead(l) if *l == c => return false,
                _ => {}
            }
        }
        let preds = &predecessors[b];
        let yes = !preds.is_empty()
            && preds.iter().all(|&p| {
                walk(
                    body,
                    region,
                    p,
                    c,
                    ty,
                    predecessors,
                    fuel,
                    depth + 1,
                    active,
                    done,
                )
            });
        active.remove(&b);
        if yes {
            done.insert(b);
        }
        yes
    }
    let mut predecessors = vec![Vec::new(); body.blocks.len()];
    for (b, data) in body.blocks.iter().enumerate() {
        for to in succ(&data.terminator) {
            predecessors[to].push(b);
        }
    }
    let preds: Vec<_> = predecessors[header]
        .iter()
        .copied()
        .filter(|p| !region.contains(p))
        .collect();
    let mut fuel = 512;
    let mut done = BTreeSet::new();
    !preds.is_empty()
        && preds.into_iter().all(|p| {
            walk(
                body,
                region,
                p,
                cursor,
                common.usize,
                &predecessors,
                &mut fuel,
                0,
                &mut BTreeSet::new(),
                &mut done,
            )
        })
}

fn operand_moves(o: &Operand, l: LocalId) -> bool {
    matches!(o,Operand::Move(p) if p.local==l)
}
fn rv_moves(r: &Rvalue, l: LocalId) -> bool {
    match r {
        Rvalue::Use(o)
        | Rvalue::Cast { operand: o, .. }
        | Rvalue::UnaryOp { operand: o, .. }
        | Rvalue::Repeat { value: o, .. } => operand_moves(o, l),
        Rvalue::BinaryOp { lhs, rhs, .. } => operand_moves(lhs, l) || operand_moves(rhs, l),
        Rvalue::Aggregate { operands, .. } => operands.iter().any(|o| operand_moves(o, l)),
        _ => false,
    }
}
fn term_moves(t: &Terminator, l: LocalId) -> bool {
    match t {
        Terminator::SwitchInt { discr, .. } => operand_moves(discr, l),
        Terminator::Call { args, func, .. } => {
            args.iter().any(|o| operand_moves(o, l))
                || matches!(func,FuncRef::Indirect{operand,..} if operand_moves(operand,l))
        }
        Terminator::Assert { cond, msg, .. } => {
            operand_moves(cond, l)
                || match msg {
                    AssertKind::Bounds { len, index } => {
                        operand_moves(len, l) || operand_moves(index, l)
                    }
                    AssertKind::RefCellBorrow { file, line } => {
                        operand_moves(file, l) || operand_moves(line, l)
                    }
                    AssertKind::Panic { message } => operand_moves(message, l),
                    _ => false,
                }
        }
        _ => false,
    }
}

fn operand_mentions(o: &Operand, l: LocalId) -> bool {
    matches!(o, Operand::Copy(p) | Operand::Move(p) if p.local == l || p.projection.iter().any(|x| matches!(x, Projection::Index(i) if *i == l)))
}
fn rv_mentions(r: &Rvalue, l: LocalId) -> bool {
    match r {
        Rvalue::Use(o)
        | Rvalue::Cast { operand: o, .. }
        | Rvalue::UnaryOp { operand: o, .. }
        | Rvalue::Repeat { value: o, .. } => operand_mentions(o, l),
        Rvalue::BinaryOp { lhs, rhs, .. } => operand_mentions(lhs, l) || operand_mentions(rhs, l),
        Rvalue::Ref { place, .. } | Rvalue::Discriminant(place) => place.local == l,
        Rvalue::Aggregate { operands, .. } => operands.iter().any(|o| operand_mentions(o, l)),
    }
}
fn term_mentions(t: &Terminator, l: LocalId) -> bool {
    match t {
        Terminator::Call { func, args, .. } => {
            args.iter().any(|o| operand_mentions(o, l))
                || matches!(func, FuncRef::Indirect { operand, .. } if operand_mentions(operand,l))
        }
        Terminator::SwitchInt { discr, .. } => operand_mentions(discr, l),
        Terminator::Assert { cond, msg, .. } => {
            operand_mentions(cond, l)
                || match msg {
                    AssertKind::Bounds { len, index } => {
                        operand_mentions(len, l) || operand_mentions(index, l)
                    }
                    AssertKind::RefCellBorrow { file, line } => {
                        operand_mentions(file, l) || operand_mentions(line, l)
                    }
                    AssertKind::Panic { message } => operand_mentions(message, l),
                    _ => false,
                }
        }
        _ => false,
    }
}

fn header_test(
    body: &Body,
    region: &BTreeSet<usize>,
    h: usize,
    defs: &Defs,
    dom: &BTreeMap<usize, BTreeSet<usize>>,
    view: LocalId,
    cursor: LocalId,
) -> Option<usize> {
    fn same_value(
        body: &Body,
        defs: &Defs,
        dom: &BTreeMap<usize, BTreeSet<usize>>,
        o: &Operand,
        at: Point,
        wanted: LocalId,
        depth: usize,
    ) -> bool {
        if depth > 16 {
            return false;
        }
        let Some(l) = local(o) else { return false };
        if l == wanted {
            return true;
        }
        let Some(p) = unique(defs, l) else {
            return false;
        };
        if !dominates(dom, p, at) {
            return false;
        }
        matches!(body.blocks[p.block].stmts.get(p.statement).map(|s| &s.kind),
            Some(StmtKind::Assign { rvalue: Rvalue::Use(from), .. }) if same_value(body,defs,dom,from,p,wanted,depth+1))
    }
    fn is_length(
        body: &Body,
        defs: &Defs,
        dom: &BTreeMap<usize, BTreeSet<usize>>,
        o: &Operand,
        at: Point,
        v: LocalId,
        depth: usize,
    ) -> bool {
        if depth > 16 {
            return false;
        }
        if matches!(o, Operand::Copy(p) if p.local == v && p.projection == vec![Projection::Field(1)])
        {
            return true;
        }
        let Some(l) = local(o) else { return false };
        let Some(p) = unique(defs, l) else {
            return false;
        };
        if !dominates(dom, p, at) {
            return false;
        }
        if let Some(Stmt {
            kind:
                StmtKind::Assign {
                    rvalue: Rvalue::Use(from),
                    ..
                },
            ..
        }) = body.blocks[p.block].stmts.get(p.statement)
        {
            return is_length(body, defs, dom, from, p, v, depth + 1);
        }
        matches!(&body.blocks[p.block].terminator,
            Terminator::Call { func: FuncRef::Builtin { which: Builtin::SpanLen, .. }, args, dest, .. }
            if p.statement == body.blocks[p.block].stmts.len() && dest.local == l && args.len()==1 && local(&args[0])==Some(v))
    }
    let mut b = h;
    let mut visited = BTreeSet::new();
    while region.contains(&b) && visited.insert(b) {
        match &body.blocks[b].terminator {
            Terminator::Goto(next)
            | Terminator::Call {
                func:
                    FuncRef::Builtin {
                        which: Builtin::SpanLen,
                        ..
                    },
                next,
                ..
            } => b = next.0 as usize,
            Terminator::SwitchInt {
                discr,
                targets,
                otherwise,
            } if targets.len() == 1
                && targets[0].0 == 0
                && !region.contains(&(targets[0].1.0 as usize))
                && region.contains(&(otherwise.0 as usize)) =>
            {
                let p = unique(defs, local(discr)?)?;
                if !dominates(
                    dom,
                    p,
                    Point {
                        block: b,
                        statement: body.blocks[b].stmts.len(),
                    },
                ) {
                    return None;
                }
                let StmtKind::Assign {
                    rvalue:
                        Rvalue::BinaryOp {
                            op: BinOp::Lt,
                            lhs,
                            rhs,
                        },
                    ..
                } = &body.blocks[p.block].stmts.get(p.statement)?.kind
                else {
                    return None;
                };
                return (same_value(body, defs, dom, lhs, p, cursor, 0)
                    && is_length(body, defs, dom, rhs, p, view, 0))
                .then_some(b);
            }
            _ => return None,
        }
    }
    None
}

struct Terms<'a> {
    body: &'a Body,
    types: &'a TypeTable,
    common: &'a CommonTypes,
    plan: &'a Plan,
    defs: &'a Defs,
    dom: &'a BTreeMap<usize, BTreeSet<usize>>,
    fuel: std::cell::Cell<usize>,
}
impl Terms<'_> {
    fn bound(
        &self,
        o: &Operand,
        at: Point,
        active: &mut BTreeSet<LocalId>,
        depth: usize,
    ) -> Option<Upper> {
        if depth > 32 {
            return None;
        }
        self.fuel.set(self.fuel.get().checked_sub(1)?);
        if let Operand::Const(Const::Int { value, ty }) = o {
            let n = constant(self.types, *value, *ty)?;
            return (0..=LIMIT as i128)
                .contains(&n)
                .then_some(Upper::Constant(n as u64));
        }
        let l = local(o)?;
        if l == self.plan.acc || l == self.plan.flag || l == self.plan.reference {
            return None;
        }
        if l == self.plan.cursor {
            let before =
                at.block == self.plan.step || !self.dom[&at.block].contains(&self.plan.step);
            return Some(if before {
                Upper::BeforeStep
            } else {
                Upper::Length
            });
        }
        if !active.insert(l) {
            return None;
        }
        let result = if self.defs.contains_key(&l) {
            let p = unique(self.defs, l)?;
            if !dominates(self.dom, p, at) {
                return None;
            }
            if let Some(s) = self.body.blocks[p.block].stmts.get(p.statement) {
                match &s.kind {
                    StmtKind::Assign {
                        rvalue: Rvalue::Use(from),
                        ..
                    } => self.bound(from, p, active, depth + 1),
                    StmtKind::Assign {
                        rvalue:
                            Rvalue::Cast {
                                kind: CastKind::Numeric | CastKind::Widen,
                                operand,
                                to,
                            },
                        ..
                    } if *to == self.body.local(l).ty => {
                        let upper = self.bound(operand, p, active, depth + 1)?;
                        let range = type_range(self.types, *to)?;
                        (range.lo <= 0 && range.hi >= upper.maximum()).then_some(upper)
                    }
                    StmtKind::Assign {
                        rvalue:
                            Rvalue::BinaryOp {
                                op: op @ (BinOp::BitAnd | BinOp::BitOr | BinOp::BitXor),
                                lhs,
                                rhs,
                            },
                        ..
                    } => {
                        if operand_ty(self.body, self.common, lhs)? != self.body.local(l).ty
                            || operand_ty(self.body, self.common, rhs)? != self.body.local(l).ty
                        {
                            return None;
                        }
                        let a = self.bound(lhs, p, active, depth + 1);
                        let b = self.bound(rhs, p, active, depth + 1);
                        match (*op, a, b) {
                            (BinOp::BitAnd, Some(a), Some(b)) => {
                                Some(Upper::Min(Box::new(a), Box::new(b)))
                            }
                            // One proven nonnegative operand clears any signed high bit.
                            (BinOp::BitAnd, Some(a), None) | (BinOp::BitAnd, None, Some(a)) => {
                                Some(a)
                            }
                            (_, Some(a), Some(b)) => {
                                let range = type_range(self.types, self.body.local(l).ty)?;
                                let max = u64::try_from(range.hi).ok()?.min(LIMIT);
                                Some(Upper::Sum(Box::new(a), Box::new(b), max))
                            }
                            _ => None,
                        }
                    }
                    _ => None,
                }
            } else if p.block == self.plan.step && self.body.local(l).ty == self.common.char_ {
                Some(Upper::Constant(0x10FFFF))
            } else {
                None
            }
        } else {
            let range = type_range(self.types, self.body.local(l).ty)?;
            (range.lo >= 0 && range.hi <= LIMIT as i128).then_some(Upper::Constant(range.hi as u64))
        };
        active.remove(&l);
        result
    }
}

fn operand_ty(body: &Body, common: &CommonTypes, o: &Operand) -> Option<Ty> {
    match o {
        Operand::Const(Const::Int { ty, .. }) => Some(*ty),
        Operand::Const(Const::Bool(_)) => Some(common.bool_),
        Operand::Copy(p) if p.projection.is_empty() => {
            body.locals.get(p.local.0 as usize).map(|l| l.ty)
        }
        Operand::Copy(p)
            if p.projection == vec![Projection::Field(1)]
                && body.local(p.local).ty == common.str_ =>
        {
            Some(common.usize)
        }
        _ => None,
    }
}
fn numeric(types: &TypeTable, common: &CommonTypes, ty: Ty) -> bool {
    ty == common.bool_ || type_range(types, ty).is_some()
}
fn typed_numeric(body: &Body, types: &TypeTable, c: &CommonTypes, dest: Ty, r: &Rvalue) -> bool {
    match r {
        Rvalue::Use(o) => operand_ty(body, c, o) == Some(dest),
        Rvalue::UnaryOp { op, operand } => {
            operand_ty(body, c, operand) == Some(dest)
                && (!matches!(op, ember_mir::UnOp::Not) || dest == c.bool_)
        }
        Rvalue::Cast {
            kind: CastKind::Numeric | CastKind::Widen,
            operand,
            to,
        } => *to == dest && operand_ty(body, c, operand).is_some_and(|t| numeric(types, c, t)),
        Rvalue::BinaryOp { op, lhs, rhs } => {
            let (Some(a), Some(b)) = (operand_ty(body, c, lhs), operand_ty(body, c, rhs)) else {
                return false;
            };
            a == b
                && numeric(types, c, a)
                && if op.is_comparison() {
                    dest == c.bool_ && !matches!(op, BinOp::Is | BinOp::IsNot)
                } else {
                    dest == a
                }
        }
        _ => false,
    }
}

fn plan(body: &Body, types: &TypeTable, common: &CommonTypes, header: usize) -> Option<Plan> {
    if !budget(body)
        || !body.call_argument_bindings.is_empty()
        || !body.hoisted_accesses.is_empty()
        || body.blocks.len() > MAX_BLOCKS - 133
        || body.locals.len() > 63000
        || body.blocks.iter().map(|b| b.stmts.len()).sum::<usize>() > MAX_STMTS / 2
        || body.ffi_counted.is_some()
        || !ember_mir::verify::verify(body).is_empty()
        || !same_body(body, body)
    {
        return None;
    }
    let order = reverse_postorder(body);
    let headers = loop_headers(body, &order);
    let region = natural_loops(body, &order, &headers).remove(&header)?;
    let region: BTreeSet<_> = region.into_iter().collect();
    if region.len() > 128 {
        return None;
    }
    // Explicit one-entry and complete exit inventory. Initial scope only normal
    // header false exit; omitted break/return tails cannot be priced as W=1.
    for &b in &region {
        if b != header
            && (0..body.blocks.len())
                .any(|p| !region.contains(&p) && succ(&body.blocks[p].terminator).contains(&b))
        {
            return None;
        }
    }
    let mut steps = Vec::new();
    let mut checks = Vec::new();
    for &b in &region {
        if matches!(
            &body.blocks[b].terminator,
            Terminator::Call {
                func: FuncRef::Builtin {
                    which: Builtin::StrCharNext,
                    ..
                },
                ..
            }
        ) {
            steps.push(b);
        }
        for (i, s) in body.blocks[b].stmts.iter().enumerate() {
            if matches!(s.kind, StmtKind::CheckedBinaryOp { .. }) {
                checks.push((b, i));
            }
        }
    }
    let [step] = steps.as_slice() else {
        return None;
    };
    let [(check, statement)] = checks.as_slice() else {
        return None;
    };
    let Terminator::Call {
        func: FuncRef::Builtin { arg_ty, .. },
        args,
        dest: character,
        ..
    } = &body.blocks[*step].terminator
    else {
        return None;
    };
    if *arg_ty != common.str_ {
        return None;
    }
    if args.len() != 2
        || !character.projection.is_empty()
        || body.local(character.local).ty != common.char_
    {
        return None;
    }
    let view = local(&args[0])?;
    if body.local(view).ty != common.str_ {
        return None;
    }
    let reference = local(&args[1])?;
    let defs = definitions(body, &region);
    let ref_point = unique(&defs, reference)?;
    let StmtKind::Assign {
        place: ref_dest,
        rvalue: Rvalue::Ref {
            place: cursor_place,
            mutable: true,
        },
    } = &body.blocks[ref_point.block]
        .stmts
        .get(ref_point.statement)?
        .kind
    else {
        return None;
    };
    if *ref_dest != Place::local(reference) {
        return None;
    }
    if !cursor_place.projection.is_empty() {
        return None;
    }
    let cursor = cursor_place.local;
    if body.local(cursor).ty != common.usize
        || !matches!(types.kind(body.local(reference).ty),
        ember_types::TyKind::Ref { mutable:true, inner } if *inner==common.usize)
    {
        return None;
    }
    let dom = dominators(body, &region, header)?;
    let test = header_test(body, &region, header, &defs, &dom, view, cursor)?;
    let StmtKind::CheckedBinaryOp {
        dest,
        overflow,
        op: BinOp::Add,
        lhs,
        rhs,
    } = &body.blocks[*check].stmts[*statement].kind
    else {
        return None;
    };
    if !dest.projection.is_empty()
        || !overflow.projection.is_empty()
        || body.local(dest.local).ty != common.i64
        || body.local(overflow.local).ty != common.bool_
    {
        return None;
    }
    let acc = dest.local;
    let flag = overflow.local;
    let term = if local(lhs) == Some(acc) {
        rhs
    } else if local(rhs) == Some(acc) {
        lhs
    } else {
        return None;
    };
    if operand_ty(body, common, term)? != common.i64 {
        return None;
    }
    if *statement + 1 != body.blocks[*check].stmts.len()
        || !matches!(&body.blocks[*check].terminator,
        Terminator::Assert { cond,expected:false,msg:AssertKind::Overflow(BinOp::Add),.. } if local(cond)==Some(flag))
    {
        return None;
    }
    if !dom[step].contains(&test)
        || !dom[check].contains(step)
        || !dominates(
            &dom,
            ref_point,
            Point {
                block: *step,
                statement: body.blocks[*step].stmts.len(),
            },
        )
    {
        return None;
    }
    // DAG after cutting all header return edges. Residual cycles/hidden nested
    // loops reject. Every latch must be reached after the semantic step.
    let mut pending = region.clone();
    let mut processed = BTreeSet::new();
    while !pending.is_empty() {
        let ready: Vec<_> = pending
            .iter()
            .copied()
            .filter(|&b| {
                b == header
                    || region.iter().all(|&p| {
                        !succ(&body.blocks[p].terminator).contains(&b) || processed.contains(&p)
                    })
            })
            .collect();
        if ready.is_empty() {
            return None;
        }
        for b in ready {
            pending.remove(&b);
            processed.insert(b);
        }
    }
    for &b in &region {
        let data = &body.blocks[b];
        for to in succ(&data.terminator) {
            if to == header && !dom[&b].contains(step) {
                return None;
            }
            if !region.contains(&to) && b != test {
                return None;
            }
        }
        for (i, s) in data.stmts.iter().enumerate() {
            match &s.kind {
                StmtKind::Assign { place, rvalue } => {
                    if !place.projection.is_empty()
                        || [view, cursor, acc, flag].contains(&place.local)
                    {
                        return None;
                    }
                    if i == ref_point.statement && b == ref_point.block { /* exactly approved ref */
                    } else {
                        if !numeric(types, common, body.local(place.local).ty) {
                            return None;
                        }
                        match rvalue {
                            Rvalue::Use(o)
                            | Rvalue::UnaryOp { operand: o, .. }
                            | Rvalue::Cast {
                                kind: CastKind::Numeric | CastKind::Widen,
                                operand: o,
                                ..
                            } => {
                                operand_ty(body, common, o)?;
                            }
                            Rvalue::BinaryOp { lhs, rhs, .. } => {
                                operand_ty(body, common, lhs)?;
                                operand_ty(body, common, rhs)?;
                            }
                            _ => return None,
                        }
                        if !typed_numeric(body, types, common, body.local(place.local).ty, rvalue) {
                            return None;
                        }
                    }
                    if rv_mentions(rvalue, acc) || rv_mentions(rvalue, flag) {
                        return None;
                    }
                }
                StmtKind::CheckedBinaryOp { .. } if b == *check && i == *statement => {}
                // The eliminated flag is never read outside its own Assert;
                // preserving its lifetime markers has no runtime effect.
                StmtKind::StorageLive(l) | StmtKind::StorageDead(l)
                    if ![view, cursor, acc].contains(l) => {}
                StmtKind::Nop => {}
                _ => return None,
            }
        }
        match &data.terminator {
            Terminator::Goto(_) | Terminator::SwitchInt { .. } => {}
            Terminator::Call {
                func:
                    FuncRef::Builtin {
                        which: Builtin::StrCharNext,
                        ..
                    },
                ..
            } if b == *step => {}
            Terminator::Call {
                func:
                    FuncRef::Builtin {
                        which: Builtin::SpanLen,
                        arg_ty,
                    },
                args,
                dest,
                ..
            } if *arg_ty == common.str_
                && args.len() == 1
                && local(&args[0]) == Some(view)
                && dest.projection.is_empty()
                && ![view, cursor, acc, flag, reference].contains(&dest.local)
                && body.local(dest.local).ty == common.usize => {}
            Terminator::Assert { .. } if b == *check => {}
            _ => return None,
        }
        if b != *check && term_mentions(&data.terminator, flag) {
            return None;
        }
    }
    // Entire-body private referent/flag inventory, not just an epoch scan.
    for (b, data) in body.blocks.iter().enumerate() {
        for (i, s) in data.stmts.iter().enumerate() {
            if matches!(&s.kind,StmtKind::Drop {place,flag:f,..} if [reference,flag,cursor].contains(&place.local) || f.is_some_and(|l| l==reference || l==flag || l==cursor))
            {
                return None;
            }
            if matches!(&s.kind,StmtKind::BeginAccess {place,..}|StmtKind::EndAccess {place,..}|StmtKind::BeginAccessTransfer {place,..}|StmtKind::EndAccessTransfer {place,..} if [reference,flag,cursor].contains(&place.local))
            {
                return None;
            }
            if let StmtKind::Assign { place, rvalue } = &s.kind {
                if let Rvalue::Ref { place: target, .. } = rvalue {
                    if [view, cursor, acc, flag].contains(&target.local)
                        && !(b == ref_point.block
                            && i == ref_point.statement
                            && target == cursor_place)
                    {
                        return None;
                    }
                }
                if place.local == reference && !(b == ref_point.block && i == ref_point.statement) {
                    return None;
                }
                if rv_mentions(rvalue, reference) || rv_mentions(rvalue, flag) {
                    return None;
                }
            }
            if let StmtKind::CheckedBinaryOp { lhs, rhs, .. } = &s.kind {
                if operand_mentions(lhs, reference)
                    || operand_mentions(rhs, reference)
                    || operand_mentions(lhs, flag)
                    || operand_mentions(rhs, flag)
                {
                    return None;
                }
            }
        }
        if term_mentions(&data.terminator, reference) && b != *step {
            return None;
        }
        if term_mentions(&data.terminator, flag) && b != *check {
            return None;
        }
        if matches!(&data.terminator,Terminator::Call {dest,..} if dest.local==reference) {
            return None;
        }
    }
    if !zero_entry(body, &region, header, cursor, common)
        || !readable(body, header, &[view, cursor, acc])
        || !reads_initialized(body, &region)
    {
        return None;
    }
    let mut result = Plan {
        header,
        region,
        test,
        step: *step,
        view,
        cursor,
        reference,
        check: *check,
        statement: *statement,
        acc,
        flag,
        upper: Upper::Constant(0),
    };
    let upper = Terms {
        body,
        types,
        common,
        plan: &result,
        defs: &defs,
        dom: &dom,
        fuel: std::cell::Cell::new(128),
    }
    .bound(
        term,
        Point {
            block: *check,
            statement: *statement,
        },
        &mut BTreeSet::new(),
        0,
    )?;
    result.upper = upper;
    Some(result)
}

struct Builder<'a> {
    body: &'a mut Body,
    common: &'a CommonTypes,
    span: Span,
}
impl Builder<'_> {
    fn temp(&mut self, ty: Ty) -> LocalId {
        let l = LocalId(self.body.locals.len() as u32);
        self.body.locals.push(LocalDecl {
            ty,
            kind: LocalKind::Temp,
            name: None,
            span: self.span,
        });
        l
    }
    fn set(&self, l: LocalId, r: Rvalue) -> Stmt {
        Stmt::new(
            StmtKind::Assign {
                place: Place::local(l),
                rvalue: r,
            },
            self.span,
        )
    }
    fn bin(&self, op: BinOp, a: Operand, b: Operand) -> Rvalue {
        Rvalue::BinaryOp { op, lhs: a, rhs: b }
    }
    fn cast(&self, o: Operand, to: Ty) -> Rvalue {
        Rvalue::Cast {
            kind: CastKind::Numeric,
            operand: o,
            to,
        }
    }
    fn block(&mut self, stmts: Vec<Stmt>, condition: LocalId, no: usize, yes: usize) -> usize {
        let b = self.body.blocks.len();
        self.body.blocks.push(BasicBlock {
            stmts,
            terminator: Terminator::SwitchInt {
                discr: copy(condition),
                targets: vec![(0, BasicBlockId(no as u32))],
                otherwise: BasicBlockId(yes as u32),
            },
            terminator_span: self.span,
        });
        b
    }
    fn upper(&mut self, u: &Upper, length: LocalId, stmts: &mut Vec<Stmt>) -> LocalId {
        let word = self.common.u64;
        let l = self.temp(word);
        match u {
            Upper::Constant(n) => stmts.push(self.set(l, Rvalue::Use(int(*n, word)))),
            Upper::Length => stmts.push(self.set(l, Rvalue::Use(copy(length)))),
            Upper::BeforeStep => {
                stmts.push(self.set(l, self.bin(BinOp::Sub, copy(length), int(1, word))))
            }
            Upper::Sum(a, b, max) => {
                let x = self.upper(a, length, stmts);
                let y = self.upper(b, length, stmts);
                stmts.push(self.set(l, self.bin(BinOp::Add, copy(x), copy(y))));
                // Pure unsigned min mask; l<=2*INTMAX. No new builtin.
                self.clamp(l, int(*max, word), stmts);
            }
            Upper::Min(a, b) => {
                let x = self.upper(a, length, stmts);
                let y = self.upper(b, length, stmts);
                stmts.push(self.set(l, Rvalue::Use(copy(x))));
                self.clamp(l, copy(y), stmts);
            }
        }
        l
    }
    fn clamp(&mut self, l: LocalId, max: Operand, stmts: &mut Vec<Stmt>) {
        // min(x,y) = y XOR ((x XOR y) AND -(u64)(x<y)); all unsigned,
        // no effect, no signed overflow, sum already bounded <=UINT64_MAX-1.
        let yes = self.temp(self.common.bool_);
        let mask = self.temp(self.common.u64);
        let different = self.temp(self.common.u64);
        let chosen = self.temp(self.common.u64);
        stmts.push(self.set(yes, self.bin(BinOp::Lt, copy(l), max.clone())));
        stmts.push(self.set(mask, self.cast(copy(yes), self.common.u64)));
        stmts.push(self.set(
            mask,
            self.bin(BinOp::Sub, int(0, self.common.u64), copy(mask)),
        ));
        stmts.push(self.set(different, self.bin(BinOp::BitXor, copy(l), max.clone())));
        stmts.push(self.set(chosen, self.bin(BinOp::BitAnd, copy(different), copy(mask))));
        stmts.push(self.set(l, self.bin(BinOp::BitXor, max, copy(chosen))));
    }
}

fn apply(body: &mut Body, common: &CommonTypes, p: &Plan) {
    let old = body.blocks.len();
    let span = body.blocks[p.header].terminator_span;
    let map: BTreeMap<_, _> = p
        .region
        .iter()
        .enumerate()
        .map(|(i, &b)| (b, old + i))
        .collect();
    for &b in &p.region {
        let mut block = body.blocks[b].clone();
        crate::loop_version::retarget(&mut block.terminator, |x| {
            BasicBlockId(*map.get(&(x.0 as usize)).unwrap_or(&(x.0 as usize)) as u32)
        });
        if b == p.check {
            let StmtKind::CheckedBinaryOp { dest, lhs, rhs, .. } =
                block.stmts[p.statement].kind.clone()
            else {
                unreachable!()
            };
            block.stmts[p.statement].kind = StmtKind::Assign {
                place: dest,
                rvalue: Rvalue::BinaryOp {
                    op: BinOp::Add,
                    lhs,
                    rhs,
                },
            };
            let Terminator::Assert { next, .. } = block.terminator else {
                unreachable!()
            };
            block.terminator = Terminator::Goto(next);
        }
        body.blocks.push(block);
    }
    let fast = map[&p.header];
    let first = body.blocks.len();
    let mut b = Builder { body, common, span };
    let native_len = b.temp(common.usize);
    let length = b.temp(common.u64);
    let cur = b.temp(common.u64);
    let runs = b.temp(common.bool_);
    let fits = b.temp(common.bool_);
    let remaining = b.temp(common.u64);
    let nonzero = b.temp(common.bool_);
    let room = b.temp(common.u64);
    let acc_bits = b.temp(common.u64);
    let q = b.temp(common.u64);
    let safe = b.temp(common.bool_);
    let mut stmts = vec![
        b.set(
            native_len,
            Rvalue::Use(Operand::Copy(Place {
                local: p.view,
                projection: vec![Projection::Field(1)],
            })),
        ),
        b.set(length, b.cast(copy(native_len), common.u64)),
        b.set(fits, b.bin(BinOp::Le, copy(length), int(LIMIT, common.u64))),
    ];
    // Lossless usize->u64 widening is needed to compare on both native32/64;
    // there is NO arithmetic and no signed narrowing before this gate.
    b.block(std::mem::take(&mut stmts), fits, p.header, first + 1);
    b.block(
        vec![b.set(runs, b.bin(BinOp::Le, copy(p.cursor), copy(native_len)))],
        runs,
        p.header,
        first + 2,
    );
    b.block(
        vec![
            b.set(cur, b.cast(copy(p.cursor), common.u64)),
            b.set(remaining, b.bin(BinOp::Sub, copy(length), copy(cur))),
            b.set(
                nonzero,
                b.bin(BinOp::Ne, copy(remaining), int(0, common.u64)),
            ),
        ],
        nonzero,
        p.header,
        first + 3,
    );
    let mut upper_stmts = Vec::new();
    let upper_bound = b.upper(&p.upper, length, &mut upper_stmts);
    let has_term = b.temp(common.bool_);
    upper_stmts.push(b.set(
        has_term,
        b.bin(BinOp::Ne, copy(upper_bound), int(0, common.u64)),
    ));
    b.block(upper_stmts, has_term, fast, first + 4);
    b.block(
        vec![
            b.set(acc_bits, b.cast(copy(p.acc), common.u64)),
            b.set(
                room,
                b.bin(BinOp::Sub, int(LIMIT, common.u64), copy(acc_bits)),
            ),
            b.set(q, b.bin(BinOp::Div, copy(room), copy(upper_bound))),
            b.set(safe, b.bin(BinOp::Le, copy(remaining), copy(q))),
        ],
        safe,
        p.header,
        fast,
    );
    for i in 0..old {
        if !p.region.contains(&i) {
            retarget(&mut b.body.blocks[i].terminator, p.header, first);
        }
    }
    b.body.removed_checks.push(RemovedCheck {
        span: b.body.blocks[p.check].terminator_span,
        kind: CheckKind::Overflow,
        proof: CheckProof::LoopEntryTest { loop_span: span },
    });
}
impl Upper {
    fn maximum(&self) -> i128 {
        match self {
            Self::Constant(n) => *n as i128,
            Self::Length | Self::BeforeStep => LIMIT as i128,
            Self::Sum(a, b, m) => (a.maximum() + b.maximum()).min(*m as i128),
            Self::Min(a, b) => a.maximum().min(b.maximum()),
        }
    }
}

// Typed equality is intentionally explicit. Opaque interface tables/FFI
// mappings are declined rather than compared through debug serialization.
// The only ignored field is callable_regions: the driver MUST regenerate
// and independently validate that derived summary after this late transform.
fn same_operand(a: &Operand, b: &Operand) -> bool {
    match (a, b) {
        (Operand::Copy(a), Operand::Copy(b)) | (Operand::Move(a), Operand::Move(b)) => a == b,
        (Operand::Const(a), Operand::Const(b)) => a == b,
        _ => false,
    }
}
fn same_operands(a: &[Operand], b: &[Operand]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(a, b)| same_operand(a, b))
}
fn same_cast(a: &CastKind, b: &CastKind) -> bool {
    matches!(
        (a, b),
        (CastKind::Numeric, CastKind::Numeric)
            | (CastKind::Widen, CastKind::Widen)
            | (CastKind::FnToC, CastKind::FnToC)
            | (CastKind::ClassUpcast, CastKind::ClassUpcast)
            | (CastKind::ClassUpcastBorrowed, CastKind::ClassUpcastBorrowed)
    )
}
fn same_rvalue(a: &Rvalue, b: &Rvalue) -> bool {
    match (a, b) {
        (Rvalue::Use(a), Rvalue::Use(b)) => same_operand(a, b),
        (
            Rvalue::BinaryOp {
                op: a,
                lhs: al,
                rhs: ar,
            },
            Rvalue::BinaryOp {
                op: b,
                lhs: bl,
                rhs: br,
            },
        ) => a == b && same_operand(al, bl) && same_operand(ar, br),
        (Rvalue::UnaryOp { op: a, operand: ao }, Rvalue::UnaryOp { op: b, operand: bo }) => {
            a == b && same_operand(ao, bo)
        }
        (
            Rvalue::Cast {
                kind: a,
                operand: ao,
                to: at,
            },
            Rvalue::Cast {
                kind: b,
                operand: bo,
                to: bt,
            },
        ) => same_cast(a, b) && at == bt && same_operand(ao, bo),
        (
            Rvalue::Aggregate {
                kind: a,
                operands: ao,
            },
            Rvalue::Aggregate {
                kind: b,
                operands: bo,
            },
        ) => a == b && same_operands(ao, bo),
        (
            Rvalue::Repeat {
                value: a,
                count: ac,
            },
            Rvalue::Repeat {
                value: b,
                count: bc,
            },
        ) => ac == bc && same_operand(a, b),
        (Rvalue::Discriminant(a), Rvalue::Discriminant(b)) => a == b,
        (
            Rvalue::Ref {
                place: a,
                mutable: am,
            },
            Rvalue::Ref {
                place: b,
                mutable: bm,
            },
        ) => a == b && am == bm,
        _ => false,
    }
}
fn same_stmt(a: &Stmt, b: &Stmt) -> bool {
    a.span == b.span
        && match (&a.kind, &b.kind) {
            (
                StmtKind::Assign {
                    place: a,
                    rvalue: ar,
                },
                StmtKind::Assign {
                    place: b,
                    rvalue: br,
                },
            ) => a == b && same_rvalue(ar, br),
            (
                StmtKind::CheckedBinaryOp {
                    dest: a,
                    overflow: af,
                    op: ao,
                    lhs: al,
                    rhs: ar,
                },
                StmtKind::CheckedBinaryOp {
                    dest: b,
                    overflow: bf,
                    op: bo,
                    lhs: bl,
                    rhs: br,
                },
            ) => a == b && af == bf && ao == bo && same_operand(al, bl) && same_operand(ar, br),
            (StmtKind::StorageLive(a), StmtKind::StorageLive(b))
            | (StmtKind::StorageDead(a), StmtKind::StorageDead(b)) => a == b,
            (
                StmtKind::Drop {
                    place: a,
                    flag: af,
                    scope_end: ae,
                },
                StmtKind::Drop {
                    place: b,
                    flag: bf,
                    scope_end: be,
                },
            ) => a == b && af == bf && ae == be,
            (
                StmtKind::BeginAccess {
                    place: a,
                    mutable: am,
                },
                StmtKind::BeginAccess {
                    place: b,
                    mutable: bm,
                },
            )
            | (
                StmtKind::EndAccess {
                    place: a,
                    mutable: am,
                },
                StmtKind::EndAccess {
                    place: b,
                    mutable: bm,
                },
            )
            | (
                StmtKind::BeginAccessTransfer {
                    place: a,
                    mutable: am,
                },
                StmtKind::BeginAccessTransfer {
                    place: b,
                    mutable: bm,
                },
            )
            | (
                StmtKind::EndAccessTransfer {
                    place: a,
                    mutable: am,
                },
                StmtKind::EndAccessTransfer {
                    place: b,
                    mutable: bm,
                },
            ) => a == b && am == bm,
            (StmtKind::Nop, StmtKind::Nop) => true,
            _ => false,
        }
}
fn same_func(a: &FuncRef, b: &FuncRef) -> bool {
    match (a, b) {
        (
            FuncRef::Direct {
                symbol: a,
                latebound: al,
            },
            FuncRef::Direct {
                symbol: b,
                latebound: bl,
            },
        ) => a == b && al == bl,
        (
            FuncRef::Builtin {
                which: a,
                arg_ty: at,
            },
            FuncRef::Builtin {
                which: b,
                arg_ty: bt,
            },
        ) => a == b && at == bt,
        (
            FuncRef::Virtual {
                owner: a,
                slot: as_,
                param_modes: am,
            },
            FuncRef::Virtual {
                owner: b,
                slot: bs,
                param_modes: bm,
            },
        ) => a == b && as_ == bs && am == bm,
        (
            FuncRef::Indirect {
                operand: a,
                ty: at,
                latebound: al,
                sources: as_,
            },
            FuncRef::Indirect {
                operand: b,
                ty: bt,
                latebound: bl,
                sources: bs,
            },
        ) => same_operand(a, b) && at == bt && al == bl && as_ == bs,
        _ => false,
    }
}
fn same_assert(a: &AssertKind, b: &AssertKind) -> bool {
    match (a, b) {
        (AssertKind::Overflow(a), AssertKind::Overflow(b)) => a == b,
        (AssertKind::DivisionByZero, AssertKind::DivisionByZero)
        | (AssertKind::SignedDivisionOverflow, AssertKind::SignedDivisionOverflow)
        | (AssertKind::ShiftTooLarge, AssertKind::ShiftTooLarge)
        | (AssertKind::Downcast, AssertKind::Downcast) => true,
        (AssertKind::Bounds { len: a, index: ai }, AssertKind::Bounds { len: b, index: bi }) => {
            same_operand(a, b) && same_operand(ai, bi)
        }
        (
            AssertKind::RefCellBorrow { file: a, line: al },
            AssertKind::RefCellBorrow { file: b, line: bl },
        ) => same_operand(a, b) && same_operand(al, bl),
        (AssertKind::Panic { message: a }, AssertKind::Panic { message: b }) => same_operand(a, b),
        _ => false,
    }
}
fn same_term(a: &Terminator, b: &Terminator) -> bool {
    match (a, b) {
        (Terminator::Goto(a), Terminator::Goto(b)) => a == b,
        (Terminator::Return, Terminator::Return)
        | (Terminator::Unreachable, Terminator::Unreachable) => true,
        (
            Terminator::SwitchInt {
                discr: a,
                targets: at,
                otherwise: ae,
            },
            Terminator::SwitchInt {
                discr: b,
                targets: bt,
                otherwise: be,
            },
        ) => same_operand(a, b) && at == bt && ae == be,
        (
            Terminator::Call {
                func: a,
                args: aa,
                dest: ad,
                next: an,
            },
            Terminator::Call {
                func: b,
                args: ba,
                dest: bd,
                next: bn,
            },
        ) => same_func(a, b) && same_operands(aa, ba) && ad == bd && an == bn,
        (
            Terminator::Assert {
                cond: a,
                expected: ae,
                msg: am,
                next: an,
                span: as_,
            },
            Terminator::Assert {
                cond: b,
                expected: be,
                msg: bm,
                next: bn,
                span: bs,
            },
        ) => same_operand(a, b) && ae == be && same_assert(am, bm) && an == bn && as_ == bs,
        _ => false,
    }
}
fn same_body(a: &Body, b: &Body) -> bool {
    a.name == b.name
        && a.symbol == b.symbol
        && a.is_unsafe == b.is_unsafe
        && a.abi == b.abi
        && a.overflow == b.overflow
        && a.fp == b.fp
        && a.inline == b.inline
        && a.export_thread_policy == b.export_thread_policy
        && a.arg_count == b.arg_count
        && a.param_modes == b.param_modes
        && a.span == b.span
        && a.borrows == b.borrows
        && a.sources == b.sources
        && a.is_lambda == b.is_lambda
        && a.emit_if_used == b.emit_if_used
        && a.borrowed_params == b.borrowed_params
        && a.call_argument_bindings == b.call_argument_bindings
        && a.for_iterators == b.for_iterators
        && a.closure_environment == b.closure_environment
        && a.closure_captures_by_move == b.closure_captures_by_move
        && a.class_owner == b.class_owner
        && a.class_virtual_slot == b.class_virtual_slot
        && a.is_abstract == b.is_abstract
        && a.is_extern_declaration == b.is_extern_declaration
        && a.ffi_counted.is_none()
        && b.ffi_counted.is_none()
        && a.mut_self == b.mut_self
        && a.elided_accesses == b.elided_accesses
        && a.hoisted_accesses == b.hoisted_accesses
        && a.uncounted_handles == b.uncounted_handles
        && a.removed_checks == b.removed_checks
        && a.restrict_views == b.restrict_views
        && a.locals.len() == b.locals.len()
        && a.locals
            .iter()
            .zip(&b.locals)
            .all(|(a, b)| a.ty == b.ty && a.kind == b.kind && a.name == b.name && a.span == b.span)
        && a.blocks.len() == b.blocks.len()
        && a.blocks.iter().zip(&b.blocks).all(|(a, b)| {
            a.terminator_span == b.terminator_span
                && a.stmts.len() == b.stmts.len()
                && a.stmts.iter().zip(&b.stmts).all(|(a, b)| same_stmt(a, b))
                && same_term(&a.terminator, &b.terminator)
        })
}

// Consume canonical fresh join facts, then check statement-point reads before
// each transfer. The initial pure scope has no moves or projected writes.
fn reads_initialized(body: &Body, blocks: &BTreeSet<usize>) -> bool {
    use crate::facts::InitializationState::{Init, Uninit};
    let facts = analyze_definite_init(body);
    if !verify_initialization_facts(body, &facts).is_empty() {
        return false;
    }
    fn place(p: &Place, s: &[crate::facts::InitializationState]) -> bool {
        s.get(p.local.0 as usize).is_some_and(|x|x.is_readable()) && p.projection.iter().all(|x|
            !matches!(x,Projection::Index(l) if !s.get(l.0 as usize).is_some_and(|v|v.is_readable())))
    }
    fn operand(o: &Operand, s: &[crate::facts::InitializationState]) -> bool {
        match o {
            Operand::Const(_) => true,
            Operand::Copy(p) => place(p, s),
            Operand::Move(_) => false,
        }
    }
    fn rv(r: &Rvalue, s: &[crate::facts::InitializationState]) -> bool {
        match r {
            Rvalue::Use(o)
            | Rvalue::UnaryOp { operand: o, .. }
            | Rvalue::Cast { operand: o, .. } => operand(o, s),
            Rvalue::BinaryOp { lhs, rhs, .. } => operand(lhs, s) && operand(rhs, s),
            Rvalue::Ref { place: p, .. } => place(p, s),
            _ => false,
        }
    }
    for &b in blocks {
        let Some(entry) = facts.block_entry(b) else {
            return false;
        };
        let mut state = entry.to_vec();
        for stmt in &body.blocks[b].stmts {
            match &stmt.kind {
                StmtKind::Assign {
                    place: p,
                    rvalue: r,
                } if p.projection.is_empty() => {
                    if !rv(r, &state) {
                        return false;
                    }
                    state[p.local.0 as usize] = Init;
                }
                StmtKind::CheckedBinaryOp {
                    dest,
                    overflow,
                    lhs,
                    rhs,
                    ..
                } if dest.projection.is_empty() && overflow.projection.is_empty() => {
                    if !operand(lhs, &state) || !operand(rhs, &state) {
                        return false;
                    }
                    state[dest.local.0 as usize] = Init;
                    state[overflow.local.0 as usize] = Init;
                }
                StmtKind::StorageLive(l) | StmtKind::StorageDead(l) => state[l.0 as usize] = Uninit,
                StmtKind::Nop => {}
                _ => return false,
            }
        }
        let yes = match &body.blocks[b].terminator {
            Terminator::Goto(_) => true,
            Terminator::Call { args, func, .. } => {
                args.iter().all(|o| operand(o, &state)) && !matches!(func, FuncRef::Indirect { .. })
            }
            Terminator::SwitchInt { discr, .. } => operand(discr, &state),
            Terminator::Assert {
                cond,
                msg: AssertKind::Overflow(_),
                ..
            } => operand(cond, &state),
            _ => false,
        };
        if !yes {
            return false;
        }
    }
    true
}

/// Initial scope: at most one certified region per body. Certificates remain
/// owned by the driver until the unconditional final verifier, never in MIR.
pub fn version_progress_reductions_all(
    bodies: &mut [Body],
    types: &TypeTable,
    common: &CommonTypes,
) -> ProgressReductionCertificates {
    let body_count = bodies.len();
    let mut certificates = Vec::new();
    for (body_index, body) in bodies.iter_mut().enumerate() {
        if !budget(body) {
            continue;
        }
        if !body.blocks.iter().any(|b| {
            matches!(
                &b.terminator,
                Terminator::Call {
                    func: FuncRef::Builtin {
                        which: Builtin::StrCharNext,
                        ..
                    },
                    ..
                }
            )
        }) {
            continue;
        }
        let order = reverse_postorder(body);
        let mut headers: Vec<_> = loop_headers(body, &order).into_iter().collect();
        headers.sort_unstable();
        if headers.len() > 64 {
            continue;
        }
        for header in headers {
            let Some(p) = plan(body, types, common, header) else {
                continue;
            };
            let certificate = ProgressReductionCertificate {
                body_index,
                original: body.clone(),
                header,
            };
            let mut candidate = body.clone();
            apply(&mut candidate, common, &p);
            // Check the enlarged body BEFORE any fresh initialization lattice
            // allocation. Over-budget candidates decline without mutation.
            if !budget(&candidate) {
                continue;
            }
            // No partial or unchecked transformation is allowed to survive.
            if let Err(message) = verify_one(&candidate, types, common, &certificate) {
                panic!("progress reduction recipe failure: {message}");
            }
            *body = candidate;
            certificates.push(certificate);
            break;
        }
    }
    ProgressReductionCertificates {
        body_count,
        planned_count: certificates.len(),
        entries: certificates,
    }
}

fn verify_one(
    actual: &Body,
    types: &TypeTable,
    common: &CommonTypes,
    c: &ProgressReductionCertificate,
) -> Result<(), String> {
    if !budget(actual) || !ember_mir::verify::verify(actual).is_empty() {
        return Err("malformed or oversized final CFG".into());
    }
    // Re-extract complete original region; a certificate supplies NO region,
    // term fact, dominator fact, alias flag or list of cloned blocks to trust.
    let p = plan(&c.original, types, common, c.header)
        .ok_or("immutable original no longer proves the semantic candidate")?;
    let old_blocks = c.original.blocks.len();
    let first = old_blocks + p.region.len();
    if actual.blocks.len() != first + 5
        || actual.locals.len() < c.original.locals.len()
        || actual.removed_checks.len() != c.original.removed_checks.len() + 1
    {
        return Err("unexpected recipe extent".into());
    }
    let mut reconstructed = actual.clone();
    reconstructed.blocks.truncate(old_blocks);
    reconstructed.locals.truncate(c.original.locals.len());
    reconstructed
        .removed_checks
        .truncate(c.original.removed_checks.len());
    for b in 0..old_blocks {
        if !p.region.contains(&b) {
            retarget(&mut reconstructed.blocks[b].terminator, first, c.header);
        }
    }
    if !same_body(&reconstructed, &c.original) {
        return Err("original statements, aliases, metadata, spans or entry edges changed".into());
    }
    let fresh = plan(&reconstructed, types, common, c.header)
        .ok_or("actual reconstructed original does not reprove complete semantic region")?;
    if fresh != p {
        return Err("actual current-CFG proof differs from immutable original".into());
    }
    if !guard_semantics(actual, common, &fresh, old_blocks, c.original.locals.len()) {
        return Err(
            "actual typed guard does not establish the unsigned bounded-reduction theorem".into(),
        );
    }
    if !clone_semantics(actual, &c.original, &fresh) {
        return Err("actual clone does not preserve the checked fallback recipe".into());
    }
    let mut expected = reconstructed;
    apply(&mut expected, common, &fresh);
    if !same_body(actual, &expected) {
        return Err("final guard, clone, fallback, locals, spans, metadata or edge differs from typed recipe".into());
    }
    if !reads_initialized(actual, &(first..first + 5).collect()) {
        return Err("new guard has an uninitialized statement-point read".into());
    }
    Ok(())
}

// Narrow independent semantic guard checker. This does NOT call Builder or
// apply. It substitutes only the new guard temporaries and matches the actual
// gates to the theorem. Shared expression nodes avoid exponential expansion
// of the min mask; no general MIR interpretation or constant optimizer.
use std::sync::Arc;
#[derive(Clone, PartialEq, Eq)]
struct GuardExpr {
    ty: Ty,
    node: GuardNode,
}
#[derive(Clone, PartialEq, Eq)]
enum GuardNode {
    Integer(u64),
    Length(LocalId),
    Cursor(LocalId),
    Acc(LocalId),
    Cast(Arc<GuardExpr>),
    Binary(BinOp, Arc<GuardExpr>, Arc<GuardExpr>),
    Min(Arc<GuardExpr>, Arc<GuardExpr>),
}
type E = Arc<GuardExpr>;
fn e(ty: Ty, node: GuardNode) -> E {
    Arc::new(GuardExpr { ty, node })
}
fn eqe(a: &E, b: &E) -> bool {
    Arc::ptr_eq(a, b) || a == b
}
fn eb(ty: Ty, op: BinOp, a: E, b: E) -> E {
    e(ty, GuardNode::Binary(op, a, b))
}
fn ec(c: &CommonTypes, n: u64) -> E {
    e(c.u64, GuardNode::Integer(n))
}
fn em(c: &CommonTypes, a: E, b: E) -> E {
    e(c.u64, GuardNode::Min(a, b))
}
fn normalize_min(x: E, c: &CommonTypes) -> E {
    // y XOR ((x XOR y) AND (0 - (u64)(x < y))). Every node must
    // have its exact unsigned/bool type; signed masks are not accepted.
    let GuardNode::Binary(BinOp::BitXor, y, selected) = &x.node else {
        return x;
    };
    let GuardNode::Binary(BinOp::BitAnd, difference, mask) = &selected.node else {
        return x;
    };
    let GuardNode::Binary(BinOp::BitXor, left, right) = &difference.node else {
        return x;
    };
    let GuardNode::Binary(BinOp::Sub, zero, cast) = &mask.node else {
        return x;
    };
    let GuardNode::Cast(test) = &cast.node else {
        return x;
    };
    let GuardNode::Binary(BinOp::Lt, lt_left, lt_right) = &test.node else {
        return x;
    };
    if [
        x.ty,
        y.ty,
        selected.ty,
        difference.ty,
        mask.ty,
        left.ty,
        right.ty,
        zero.ty,
        cast.ty,
    ]
    .iter()
    .any(|&t| t != c.u64)
        || test.ty != c.bool_
        || !matches!(zero.node, GuardNode::Integer(0))
        || !eqe(y, right)
        || !eqe(left, lt_left)
        || !eqe(right, lt_right)
    {
        return x;
    }
    em(c, left.clone(), right.clone())
}
fn upper_expr(u: &Upper, length: &E, c: &CommonTypes) -> E {
    match u {
        Upper::Constant(n) => ec(c, *n),
        Upper::Length => length.clone(),
        Upper::BeforeStep => eb(c.u64, BinOp::Sub, length.clone(), ec(c, 1)),
        Upper::Sum(a, b, max) => em(
            c,
            eb(
                c.u64,
                BinOp::Add,
                upper_expr(a, length, c),
                upper_expr(b, length, c),
            ),
            ec(c, *max),
        ),
        Upper::Min(a, b) => em(c, upper_expr(a, length, c), upper_expr(b, length, c)),
    }
}
fn guard_semantics(
    actual: &Body,
    c: &CommonTypes,
    p: &Plan,
    old_blocks: usize,
    old_locals: usize,
) -> bool {
    let first = old_blocks + p.region.len();
    let Some(pos) = p.region.iter().position(|&b| b == p.header) else {
        return false;
    };
    let fast = old_blocks + pos;
    if actual.blocks.len() != first + 5 {
        return false;
    }
    let native = e(c.usize, GuardNode::Length(p.view));
    let length = e(c.u64, GuardNode::Cast(native.clone()));
    let cursor = e(c.usize, GuardNode::Cursor(p.cursor));
    let cur = e(c.u64, GuardNode::Cast(cursor.clone()));
    let remaining = eb(c.u64, BinOp::Sub, length.clone(), cur);
    let upper = upper_expr(&p.upper, &length, c);
    let acc = e(c.u64, GuardNode::Cast(e(c.i64, GuardNode::Acc(p.acc))));
    let room = eb(c.u64, BinOp::Sub, ec(c, LIMIT), acc);
    let quotient = eb(c.u64, BinOp::Div, room.clone(), upper.clone());
    let conditions = [
        eb(c.bool_, BinOp::Le, length.clone(), ec(c, LIMIT)),
        eb(c.bool_, BinOp::Le, cursor, native),
        eb(c.bool_, BinOp::Ne, remaining.clone(), ec(c, 0)),
        eb(c.bool_, BinOp::Ne, upper.clone(), ec(c, 0)),
        eb(c.bool_, BinOp::Le, remaining, quotient),
    ];
    let edges = [
        (p.header, first + 1),
        (p.header, first + 2),
        (p.header, first + 3),
        (fast, first + 4),
        (p.header, fast),
    ];
    let mut values = BTreeMap::<LocalId, E>::new();
    for phase in 0..5 {
        let operand = |o: &Operand, values: &BTreeMap<LocalId, E>| -> Option<E> {
            match o {
                Operand::Const(Const::Int { value, ty })
                    if *ty == c.u64 && *value <= u64::MAX as u128 =>
                {
                    Some(ec(c, *value as u64))
                }
                Operand::Copy(place)
                    if place.local == p.view
                        && place.projection == vec![Projection::Field(1)]
                        && phase == 0 =>
                {
                    Some(e(c.usize, GuardNode::Length(p.view)))
                }
                Operand::Copy(place) if place.projection.is_empty() && place.local == p.cursor => {
                    Some(e(c.usize, GuardNode::Cursor(p.cursor)))
                }
                Operand::Copy(place)
                    if place.projection.is_empty() && place.local == p.acc && phase == 4 =>
                {
                    Some(e(c.i64, GuardNode::Acc(p.acc)))
                }
                Operand::Copy(place)
                    if place.projection.is_empty() && place.local.0 as usize >= old_locals =>
                {
                    values.get(&place.local).cloned()
                }
                _ => None,
            }
        };
        for stmt in &actual.blocks[first + phase].stmts {
            let StmtKind::Assign { place, rvalue } = &stmt.kind else {
                return false;
            };
            if !place.projection.is_empty() || (place.local.0 as usize) < old_locals {
                return false;
            }
            let ty = actual.local(place.local).ty;
            let value = match rvalue {
                Rvalue::Use(o) => {
                    let Some(a) = operand(o, &values) else {
                        return false;
                    };
                    if ty != a.ty {
                        return false;
                    }
                    a
                }
                Rvalue::Cast {
                    kind: CastKind::Numeric | CastKind::Widen,
                    operand: o,
                    to,
                } => {
                    let Some(a) = operand(o, &values) else {
                        return false;
                    };
                    if *to != c.u64
                        || ty != c.u64
                        || ![c.usize, c.u64, c.bool_, c.i64].contains(&a.ty)
                    {
                        return false;
                    }
                    if a.ty == c.i64 && phase != 4 {
                        return false;
                    }
                    e(c.u64, GuardNode::Cast(a))
                }
                Rvalue::BinaryOp { op, lhs, rhs } => {
                    let (Some(a), Some(b)) = (operand(lhs, &values), operand(rhs, &values)) else {
                        return false;
                    };
                    if a.ty != b.ty {
                        return false;
                    }
                    let allowed = match phase {
                        0 | 1 => matches!(op, BinOp::Le),
                        2 => matches!(op, BinOp::Sub | BinOp::Ne),
                        3 => matches!(
                            op,
                            BinOp::Add
                                | BinOp::Sub
                                | BinOp::Lt
                                | BinOp::BitXor
                                | BinOp::BitAnd
                                | BinOp::Ne
                        ),
                        4 => matches!(op, BinOp::Sub | BinOp::Div | BinOp::Le),
                        _ => false,
                    };
                    if !allowed {
                        return false;
                    }
                    if *op == BinOp::Div && (!eqe(&a, &room) || !eqe(&b, &upper)) {
                        return false;
                    }
                    let comparison = matches!(op, BinOp::Le | BinOp::Lt | BinOp::Ne);
                    if comparison {
                        if ty != c.bool_ || ![c.u64, c.usize].contains(&a.ty) {
                            return false;
                        }
                    } else if ty != c.u64 || a.ty != c.u64 {
                        return false;
                    }
                    normalize_min(eb(ty, *op, a, b), c)
                }
                _ => return false,
            };
            values.insert(place.local, value);
        }
        let Terminator::SwitchInt {
            discr,
            targets,
            otherwise,
        } = &actual.blocks[first + phase].terminator
        else {
            return false;
        };
        let Some(condition) = operand(discr, &values) else {
            return false;
        };
        if !eqe(&condition, &conditions[phase])
            || targets != &vec![(0, BasicBlockId(edges[phase].0 as u32))]
            || otherwise.0 as usize != edges[phase].1
        {
            return false;
        }
    }
    true
}
fn clone_semantics(actual: &Body, original: &Body, p: &Plan) -> bool {
    let map: BTreeMap<_, _> = p
        .region
        .iter()
        .enumerate()
        .map(|(i, &b)| (b, original.blocks.len() + i))
        .collect();
    for (&old, &new) in &map {
        let before = &original.blocks[old];
        let after = &actual.blocks[new];
        if before.stmts.len() != after.stmts.len()
            || before.terminator_span != after.terminator_span
        {
            return false;
        }
        for (i, (a, b)) in before.stmts.iter().zip(&after.stmts).enumerate() {
            if old == p.check && i == p.statement {
                let (
                    StmtKind::CheckedBinaryOp {
                        dest,
                        op: BinOp::Add,
                        lhs,
                        rhs,
                        ..
                    },
                    StmtKind::Assign {
                        place,
                        rvalue:
                            Rvalue::BinaryOp {
                                op: BinOp::Add,
                                lhs: al,
                                rhs: ar,
                            },
                    },
                ) = (&a.kind, &b.kind)
                else {
                    return false;
                };
                if dest != place
                    || a.span != b.span
                    || !same_operand(lhs, al)
                    || !same_operand(rhs, ar)
                {
                    return false;
                }
            } else if !same_stmt(a, b) {
                return false;
            }
        }
        let mut term = before.terminator.clone();
        if old == p.check {
            let Terminator::Assert { next, .. } = term else {
                return false;
            };
            term = Terminator::Goto(next);
        }
        crate::loop_version::retarget(&mut term, |b| {
            BasicBlockId(map.get(&(b.0 as usize)).copied().unwrap_or(b.0 as usize) as u32)
        });
        if !same_term(&term, &after.terminator) {
            return false;
        }
    }
    true
}

/// Unconditional final proof gate. Run after refreshed summaries and directly
/// before for_codegen; any violation is a compiler error, never an excuse to
/// emit the mutated fast body. No debug-mode gating or certificate checksum.
pub fn verify_progress_reductions_all(
    bodies: &[Body],
    types: &TypeTable,
    common: &CommonTypes,
    certificates: &ProgressReductionCertificates,
) -> Vec<ember_mir::verify::Violation> {
    let mut violations = Vec::new();
    let mut seen = BTreeSet::new();
    if certificates.body_count != bodies.len()
        || certificates.planned_count != certificates.entries.len()
    {
        violations.push(ember_mir::verify::Violation {
            body: "<progress-proof-batch>".into(),
            message: "changed body count or incomplete immutable proof batch".into(),
        });
    }
    for c in &certificates.entries {
        let error = if !seen.insert(c.body_index) {
            Some("duplicate body certificate".to_string())
        } else if let Some(body) = bodies.get(c.body_index) {
            verify_one(body, types, common, c).err()
        } else {
            Some("missing certified final body".to_string())
        };
        if let Some(message) = error {
            violations.push(ember_mir::verify::Violation {
                body: c.original.symbol.clone(),
                message,
            });
        }
    }
    violations
}

#[cfg(test)]
#[path = "progress_reduction_tests.rs"]
mod tests;
