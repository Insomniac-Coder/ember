//! Strength reduction in counted loops.
//!
//! A value a counted loop computes each turn as the counter times a number
//! the loop does not change, plus another (`k * step + skip`, a range's
//! `start + step * k`, an `enumerate` number `start + k`), gets a running
//! value of its own: set before the loop from the counter's value there,
//! advanced by its stride in the loop's step (which a `continue` runs too),
//! and read where the value was computed. Hand-written C keeps such an index
//! in a variable of its own, and the C compilers vectorise it as one; rebuilt
//! from the counter every turn, clang spends a multiply and an add on it in
//! every lane (the stepped range benchmark: 1.26x C, and C's speed with it).
//!
//! Only a loop a vectoriser can take is reduced: one with no check left, no
//! call and no division a vector instruction set lacks (`[SIMD-5]`). One that
//! runs a turn at a time gains nothing (a multiply by a number is as cheap as
//! an add) and carries one more value between turns.
//!
//! Only 64-bit values are reduced, all in 64-bit unsigned arithmetic, which
//! wraps: a running value is then exactly the value the loop computed, and
//! the one step it takes past the last turn cannot overflow. A checked
//! operation (`CheckedBinaryOp`) is never reduced, so no check is lost.

use std::collections::{HashMap, HashSet};

use ember_mir::{
    AssertKind, BasicBlock, BasicBlockId, BinOp, Body, Builtin, CastKind, Const, FuncRef, LocalDecl, LocalId,
    LocalKind, Operand, Place, Projection, Rvalue, Stmt, StmtKind, Terminator,
};
use ember_types::{CommonTypes, Ty, TypeTable};

use crate::loop_version::{CountedLoop, counted_loop, retarget, scalar_only};
use crate::regions::place_type;

/// Reduce the counter-derived values of every counted loop; returns how many
/// running values were made.
pub fn reduce_induction_values_all(bodies: &mut [Body], types: &TypeTable, common: &CommonTypes) -> usize {
    bodies.iter_mut().map(|body| reduce_body(body, types, common)).sum()
}

fn reduce_body(body: &mut Body, types: &TypeTable, common: &CommonTypes) -> usize {
    // A reduced loop gains a block before it and statements in its step, and
    // keeps every block's number, so each header is looked at once.
    let headers = body.blocks.len();
    (0..headers)
        .map(|header| match counted_loop(body, types, header) {
            Some(shape) => reduce_loop(body, types, common, &shape),
            None => 0,
        })
        .sum()
}

/// `count * m + b` over 64-bit wrapping arithmetic, each side a sum and
/// product of numbers the loop does not change.
#[derive(Clone)]
enum Lin {
    Const(u64),
    Leaf(Operand),
    Add(Box<Lin>, Box<Lin>),
    Sub(Box<Lin>, Box<Lin>),
    Mul(Box<Lin>, Box<Lin>),
}

impl Lin {
    fn add(a: Lin, b: Lin) -> Lin {
        match (a, b) {
            (Lin::Const(0), x) | (x, Lin::Const(0)) => x,
            (Lin::Const(x), Lin::Const(y)) => Lin::Const(x.wrapping_add(y)),
            (a, b) => Lin::Add(Box::new(a), Box::new(b)),
        }
    }

    fn sub(a: Lin, b: Lin) -> Lin {
        match (a, b) {
            (x, Lin::Const(0)) => x,
            (Lin::Const(x), Lin::Const(y)) => Lin::Const(x.wrapping_sub(y)),
            (a, b) => Lin::Sub(Box::new(a), Box::new(b)),
        }
    }

    fn mul(a: Lin, b: Lin) -> Lin {
        match (a, b) {
            (Lin::Const(0), _) | (_, Lin::Const(0)) => Lin::Const(0),
            (Lin::Const(1), x) | (x, Lin::Const(1)) => x,
            (Lin::Const(x), Lin::Const(y)) => Lin::Const(x.wrapping_mul(y)),
            (a, b) => Lin::Mul(Box::new(a), Box::new(b)),
        }
    }
}

#[derive(Clone)]
struct Affine {
    m: Lin,
    b: Lin,
}

impl Affine {
    fn constant(b: Lin) -> Affine {
        Affine { m: Lin::Const(0), b }
    }

    fn invariant(&self) -> bool {
        matches!(self.m, Lin::Const(0))
    }
}

/// A value to give a running value: where it is computed and what it is.
struct Candidate {
    block: usize,
    /// The statement's index, or `None` for the block's terminator (a
    /// range's value, `RangeNth`).
    stmt: Option<usize>,
    dest: LocalId,
    value: Affine,
}

fn reduce_loop(body: &mut Body, types: &TypeTable, common: &CommonTypes, shape: &CountedLoop) -> usize {
    // Only a loop a vectoriser can take: no check left, no call but a range's
    // value, and no division no vector instruction set does (`[SIMD-5]`,
    // ODR-088). In a loop that runs one turn at a time, the value's multiply
    // is as cheap as the running value's add, and the running value is one
    // more thing carried from turn to turn (integer arithmetic, whose `%` and
    // `//` keep it scalar: 7% slower on clang with it).
    let vectorisable = !scalar_only(body, types, shape) && shape.region.iter().all(|&block| {
        let data = &body.blocks[block];
        !data.stmts.iter().any(|stmt| matches!(stmt.kind, StmtKind::CheckedBinaryOp { .. }))
            && match &data.terminator {
                Terminator::Goto(_) | Terminator::SwitchInt { .. } => true,
                Terminator::Call { func: FuncRef::Builtin { which: Builtin::RangeNth, .. }, .. } => true,
                _ => false,
            }
    });
    if !vectorisable {
        return 0;
    }
    let counter = shape.counter;
    let mut inside: Vec<usize> = shape.region.clone();
    inside.push(shape.header);
    // What the loop writes, and what anything anywhere takes the address of:
    // neither is a number the loop does not change.
    let mut writes: HashMap<LocalId, usize> = HashMap::new();
    let mut scoped: HashSet<LocalId> = HashSet::new();
    for &block in &inside {
        for stmt in &body.blocks[block].stmts {
            for local in written(stmt) {
                *writes.entry(local).or_default() += 1;
            }
            if let StmtKind::StorageLive(local) | StmtKind::StorageDead(local) = stmt.kind {
                scoped.insert(local);
            }
        }
        if let Terminator::Call { dest, .. } = &body.blocks[block].terminator {
            *writes.entry(dest.local).or_default() += 1;
        }
    }
    if writes.get(&counter).copied().unwrap_or(0) != 1 {
        return 0;
    }
    let mut addressed = vec![false; body.locals.len()];
    for block in &body.blocks {
        for stmt in &block.stmts {
            if let StmtKind::Assign { rvalue: Rvalue::Ref { place, .. }, .. } = &stmt.kind {
                addressed[place.local.0 as usize] = true;
            }
        }
    }
    let unchanged = |place: &Place| {
        !writes.contains_key(&place.local)
            && !scoped.contains(&place.local)
            && !addressed[place.local.0 as usize]
            && place.projection.iter().all(|step| matches!(step, Projection::Field(_)))
    };
    let wide = |ty: Ty| types.is_integral(ty) && ember_types::bit_width(types, ty) == Some(64);
    let classify = |operand: &Operand, known: &HashMap<LocalId, Affine>| -> Option<Affine> {
        match operand {
            Operand::Const(Const::Int { value: 0, .. }) => Some(Affine::constant(Lin::Const(0))),
            Operand::Const(Const::Int { value: 1, .. }) => Some(Affine::constant(Lin::Const(1))),
            Operand::Const(Const::Int { .. }) => Some(Affine::constant(Lin::Leaf(operand.clone()))),
            Operand::Copy(place) | Operand::Move(place) => {
                if place.projection.is_empty() {
                    if place.local == counter {
                        return Some(Affine { m: Lin::Const(1), b: Lin::Const(0) });
                    }
                    if let Some(value) = known.get(&place.local) {
                        return Some(value.clone());
                    }
                }
                let ty = place_type(body, types, place);
                (unchanged(place) && types.is_integral(ty))
                    .then(|| Affine::constant(Lin::Leaf(Operand::Copy(place.clone()))))
            }
            _ => None,
        }
    };

    // The values, block by block in the loop's body: through copies, casts
    // between 64-bit types, `+`, `-` and `*` by an unchanged number, and a
    // range's `RangeNth`. A value is followed within its block, where its
    // one definition precedes its uses.
    let mut candidates: Vec<Candidate> = Vec::new();
    for &block in &shape.region {
        if block == shape.step {
            continue;
        }
        let mut known: HashMap<LocalId, Affine> = HashMap::new();
        for (index, stmt) in body.blocks[block].stmts.iter().enumerate() {
            let StmtKind::Assign { place, rvalue } = &stmt.kind else {
                for local in written(stmt) {
                    known.remove(&local);
                }
                if let StmtKind::StorageDead(local) = stmt.kind {
                    known.remove(&local);
                }
                continue;
            };
            known.remove(&place.local);
            let dest = place.local;
            let defined_once = place.projection.is_empty() && writes.get(&dest) == Some(&1);
            if !defined_once || !wide(body.local(dest).ty) {
                continue;
            }
            let value = match rvalue {
                Rvalue::Use(operand) => classify(operand, &known),
                Rvalue::Cast { kind: CastKind::Numeric, operand, .. } => classify(operand, &known),
                Rvalue::BinaryOp { op: op @ (BinOp::Add | BinOp::Sub | BinOp::Mul), lhs, rhs } => {
                    match (classify(lhs, &known), classify(rhs, &known)) {
                        (Some(a), Some(b)) => combine(*op, a, b),
                        _ => None,
                    }
                }
                _ => None,
            };
            let Some(value) = value else { continue };
            if matches!(rvalue, Rvalue::BinaryOp { .. }) && !value.invariant() {
                candidates.push(Candidate { block, stmt: Some(index), dest, value: value.clone() });
            }
            known.insert(dest, value);
        }
        // `(T)(start + step * k)`, in 64 bits.
        if let Terminator::Call { func: FuncRef::Builtin { which: Builtin::RangeNth, .. }, args, dest, .. } =
            &body.blocks[block].terminator
            && dest.projection.is_empty()
            && writes.get(&dest.local) == Some(&1)
            && wide(body.local(dest.local).ty)
            && let [start, step, at] = args.as_slice()
            && let (Some(start), Some(step), Some(at)) = (classify(start, &known), classify(step, &known), classify(at, &known))
            && start.invariant()
            && step.invariant()
            && !at.invariant()
        {
            let value = Affine { m: Lin::mul(step.b.clone(), at.m), b: Lin::add(start.b, Lin::mul(step.b, at.b)) };
            candidates.push(Candidate { block, stmt: None, dest: dest.local, value });
        }
    }
    // Only a value something else reads: one read only to compute another
    // value here is folded into that one's running value.
    let computed: Vec<(usize, Option<usize>)> = candidates.iter().map(|c| (c.block, c.stmt)).collect();
    let mut read = vec![false; body.locals.len()];
    for (b, block) in body.blocks.iter().enumerate() {
        for (s, stmt) in block.stmts.iter().enumerate() {
            if !computed.contains(&(b, Some(s))) {
                stmt_reads(stmt, &mut read);
            }
        }
        if !computed.contains(&(b, None)) {
            terminator_reads(&block.terminator, &mut read);
        }
    }
    candidates.retain(|candidate| read[candidate.dest.0 as usize]);
    if candidates.is_empty() {
        return 0;
    }

    // Before the loop: each running value from the counter's value there.
    let u64_ty = common.u64;
    let span = body.blocks[shape.header].terminator_span;
    let mut temp = |body: &mut Body| {
        body.locals.push(LocalDecl { ty: u64_ty, kind: LocalKind::Temp, name: None, span });
        LocalId(body.locals.len() as u32 - 1)
    };
    let mut before = Vec::new();
    let mut steps = Vec::new();
    let mut running = Vec::new();
    let count = Lin::Leaf(Operand::Copy(Place::local(counter)));
    for candidate in &candidates {
        let stride = materialize(&candidate.value.m, body, &mut before, u64_ty, span, &mut temp);
        let stride_local = temp(body);
        before.push(assign(stride_local, Rvalue::Use(stride), span));
        let start = Lin::add(Lin::mul(count.clone(), candidate.value.m.clone()), candidate.value.b.clone());
        let start = materialize(&start, body, &mut before, u64_ty, span, &mut temp);
        let value = temp(body);
        before.push(assign(value, Rvalue::Use(start), span));
        steps.push(assign(
            value,
            Rvalue::BinaryOp { op: BinOp::Add, lhs: Operand::Copy(Place::local(value)), rhs: Operand::Copy(Place::local(stride_local)) },
            span,
        ));
        running.push(value);
    }
    let header = BasicBlockId(shape.header as u32);
    let preheader = BasicBlockId(body.blocks.len() as u32);
    for block in 0..body.blocks.len() {
        if block != shape.header && !shape.region.contains(&block) {
            retarget(&mut body.blocks[block].terminator, |target| if target == header { preheader } else { target });
        }
    }
    body.blocks.push(BasicBlock { stmts: before, terminator: Terminator::Goto(header), terminator_span: span });
    // In the step, after the counter's increment.
    body.blocks[shape.step].stmts.extend(steps);
    // Each value read from its running value.
    for (candidate, value) in candidates.iter().zip(running) {
        let ty = body.local(candidate.dest).ty;
        let read = if ty == u64_ty {
            Rvalue::Use(Operand::Copy(Place::local(value)))
        } else {
            Rvalue::Cast { kind: CastKind::Numeric, operand: Operand::Copy(Place::local(value)), to: ty }
        };
        let data = &mut body.blocks[candidate.block];
        match candidate.stmt {
            Some(index) => {
                data.stmts[index].kind = StmtKind::Assign { place: Place::local(candidate.dest), rvalue: read };
            }
            None => {
                let Terminator::Call { next, .. } = data.terminator else { unreachable!("a RangeNth call") };
                data.stmts.push(assign(candidate.dest, read, span));
                data.terminator = Terminator::Goto(next);
            }
        }
    }
    remove_dead_temps(body, &shape.region);
    candidates.len()
}

/// The steps that computed a reduced value from the counter are now read by
/// nothing: remove them, in rounds, since each can leave the one before it
/// unread. MSVC vectorises no loop that sets a variable it cannot prove dead
/// after the loop, and an unread temporary is declared for the whole
/// function.
fn remove_dead_temps(body: &mut Body, region: &[usize]) {
    loop {
        let mut read = vec![false; body.locals.len()];
        for block in &body.blocks {
            block.stmts.iter().for_each(|stmt| stmt_reads(stmt, &mut read));
            terminator_reads(&block.terminator, &mut read);
        }
        let mut removed = false;
        for &block in region {
            for stmt in &mut body.blocks[block].stmts {
                let StmtKind::Assign { place, rvalue: Rvalue::Use(_) | Rvalue::Cast { .. } | Rvalue::BinaryOp { .. } } = &stmt.kind
                else {
                    continue;
                };
                let local = place.local;
                if place.projection.is_empty() && !read[local.0 as usize] && body.locals[local.0 as usize].kind == LocalKind::Temp {
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

fn combine(op: BinOp, a: Affine, b: Affine) -> Option<Affine> {
    Some(match op {
        BinOp::Add => Affine { m: Lin::add(a.m, b.m), b: Lin::add(a.b, b.b) },
        BinOp::Sub => Affine { m: Lin::sub(a.m, b.m), b: Lin::sub(a.b, b.b) },
        BinOp::Mul if a.invariant() => Affine { m: Lin::mul(a.b.clone(), b.m), b: Lin::mul(a.b, b.b) },
        BinOp::Mul if b.invariant() => Affine { m: Lin::mul(a.m, b.b.clone()), b: Lin::mul(a.b, b.b) },
        _ => return None,
    })
}

fn assign(local: LocalId, rvalue: Rvalue, span: ember_span::Span) -> Stmt {
    Stmt::new(StmtKind::Assign { place: Place::local(local), rvalue }, span)
}

/// `lin` as an operand, the statements that compute it appended to `out`.
fn materialize(
    lin: &Lin,
    body: &mut Body,
    out: &mut Vec<Stmt>,
    u64_ty: Ty,
    span: ember_span::Span,
    temp: &mut impl FnMut(&mut Body) -> LocalId,
) -> Operand {
    match lin {
        Lin::Const(value) => Operand::Const(Const::Int { value: u128::from(*value), ty: u64_ty }),
        Lin::Leaf(operand) => {
            let local = temp(body);
            out.push(assign(local, Rvalue::Cast { kind: CastKind::Numeric, operand: operand.clone(), to: u64_ty }, span));
            Operand::Copy(Place::local(local))
        }
        Lin::Add(a, b) | Lin::Sub(a, b) | Lin::Mul(a, b) => {
            let op = match lin {
                Lin::Add(..) => BinOp::Add,
                Lin::Sub(..) => BinOp::Sub,
                _ => BinOp::Mul,
            };
            let lhs = materialize(a, body, out, u64_ty, span, temp);
            let rhs = materialize(b, body, out, u64_ty, span, temp);
            let local = temp(body);
            out.push(assign(local, Rvalue::BinaryOp { op, lhs, rhs }, span));
            Operand::Copy(Place::local(local))
        }
    }
}

/// The locals a statement writes a value to.
fn written(stmt: &Stmt) -> Vec<LocalId> {
    match &stmt.kind {
        StmtKind::Assign { place, .. } => vec![place.local],
        StmtKind::CheckedBinaryOp { dest, overflow, .. } => vec![dest.local, overflow.local],
        StmtKind::Drop { place, .. } => vec![place.local],
        _ => Vec::new(),
    }
}

fn operand_reads(operand: &Operand, read: &mut [bool]) {
    if let Operand::Copy(place) | Operand::Move(place) = operand {
        place_reads(place, read);
    }
}

fn place_reads(place: &Place, read: &mut [bool]) {
    read[place.local.0 as usize] = true;
    for step in &place.projection {
        if let Projection::Index(local) = step {
            read[local.0 as usize] = true;
        }
    }
}

/// Every local a statement reads, or names in a place it writes through.
pub(crate) fn stmt_reads(stmt: &Stmt, read: &mut [bool]) {
    match &stmt.kind {
        StmtKind::Assign { place, rvalue } => {
            if !place.projection.is_empty() {
                place_reads(place, read);
            }
            match rvalue {
                Rvalue::Use(operand) | Rvalue::UnaryOp { operand, .. } | Rvalue::Cast { operand, .. } => {
                    operand_reads(operand, read);
                }
                Rvalue::BinaryOp { lhs, rhs, .. } => {
                    operand_reads(lhs, read);
                    operand_reads(rhs, read);
                }
                Rvalue::Aggregate { operands, .. } => operands.iter().for_each(|operand| operand_reads(operand, read)),
                Rvalue::Repeat { value, .. } => operand_reads(value, read),
                Rvalue::Discriminant(place) | Rvalue::Ref { place, .. } => place_reads(place, read),
            }
        }
        StmtKind::CheckedBinaryOp { dest, overflow, lhs, rhs, .. } => {
            place_reads(dest, read);
            place_reads(overflow, read);
            operand_reads(lhs, read);
            operand_reads(rhs, read);
        }
        StmtKind::BeginAccess { place, .. }
        | StmtKind::EndAccess { place, .. }
        | StmtKind::BeginAccessTransfer { place, .. }
        | StmtKind::EndAccessTransfer { place, .. } => place_reads(place, read),
        // A drop reads its flag: whether the value is still there.
        StmtKind::Drop { place, flag, .. } => {
            place_reads(place, read);
            if let Some(flag) = flag {
                read[flag.0 as usize] = true;
            }
        }
        StmtKind::StorageLive(_) | StmtKind::StorageDead(_) | StmtKind::Nop => {}
    }
}

pub(crate) fn terminator_reads(terminator: &Terminator, read: &mut [bool]) {
    match terminator {
        Terminator::SwitchInt { discr, .. } => operand_reads(discr, read),
        Terminator::Call { func, args, dest, .. } => {
            if let FuncRef::Indirect { operand, .. } = func {
                operand_reads(operand, read);
            }
            args.iter().for_each(|arg| operand_reads(arg, read));
            if !dest.projection.is_empty() {
                place_reads(dest, read);
            }
        }
        Terminator::Assert { cond, msg, .. } => {
            operand_reads(cond, read);
            match msg {
                AssertKind::Bounds { len, index } => {
                    operand_reads(len, read);
                    operand_reads(index, read);
                }
                AssertKind::RefCellBorrow { file, line } => {
                    operand_reads(file, read);
                    operand_reads(line, read);
                }
                AssertKind::Panic { message } => operand_reads(message, read),
                _ => {}
            }
        }
        Terminator::Return => read[0] = true,
        Terminator::Goto(_) | Terminator::Unreachable => {}
    }
}
