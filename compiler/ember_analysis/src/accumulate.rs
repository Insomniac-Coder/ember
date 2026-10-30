//! A function whose recursive case returns a call of itself added to a value,
//! `return fib(n - 1) + fib(n - 2)`, runs that call as a turn of a loop that adds the value into a
//! running total: one call a turn where there were two. A C compiler does this to C's `+`, which
//! cannot fail (clang to the C twin of `tests/perf/a10_recursion.em`), and not to a checked `+`,
//! whose check sits between the addition and the return. Here it is done before either backend,
//! keeping every check (`[TYP-8]`): each addition into the total is checked, and so is the return's.
//! On an overflow the turn makes the call it stands for, as the function would have made it (the
//! rewritten function makes the calls the original did), and then panics with the written `+`'s
//! message, where the function would have: after its last call. So a turn pays for its check what
//! the function did, one jump on the overflow flag, into a path that ends in the cold panic.
//!
//! The additions are grouped otherwise, `a + (b + (c + d))` as `((a + b) + c) + d`: the same sum
//! when nothing overflows. When no value added is negative (an unsigned number never is), one
//! grouping overflows exactly when the other does, since each partial sum of either lies between 0
//! and the whole sum; so the program panics in exactly the cases it did. That is proved from the
//! range facts (`range_facts.rs`): the arguments the function passes itself bound its parameters,
//! and within those bounds every value it returns is at least 0, a value it adds from a call of
//! itself by the same proof. The calls are made in the order they were, so whatever they do happens
//! as it did, and the function's frame holds only numbers and `bool`s, so it has nothing to drop.

use std::collections::{BTreeMap, HashMap};

use ember_mir::{
    AssertKind, BasicBlock, BasicBlockId, BinOp, Body, Const, FuncRef, LocalDecl, LocalId, LocalKind, Operand, Place, Rvalue, Stmt,
    StmtKind, Terminator,
};
use ember_span::Span;
use ember_types::{CommonTypes, TyKind, TypeTable};

use crate::range_facts::{field_element_ranges, returned_arguments, rvalue_operands, Analysis, FieldKey, Interval};

/// Every body of that shape rewritten so; how many were. `test_at_end`: the loop tests whether to
/// go on at the end of a turn, not at its start (see `rewrite`).
pub fn accumulate_recursive_sums_all(bodies: &mut [Body], types: &TypeTable, common: &CommonTypes, test_at_end: bool) -> usize {
    let returns = returned_arguments(bodies);
    let fields = field_element_ranges(bodies, types, common, &returns);
    let mut rewritten = 0;
    for body in bodies.iter_mut() {
        if let Some(shape) = shape(body, types, common, &returns, &fields) {
            rewrite(body, shape, test_at_end);
            rewritten += 1;
        }
    }
    rewritten
}

/// What the rewrite needs of a body of the shape.
struct Shape {
    /// The block whose call of the function itself becomes the loop's turn.
    tail: usize,
    /// That call's arguments: the parameters of the next turn.
    args: Vec<Operand>,
    /// The value the turn adds into the total: the addition's other operand.
    added: Operand,
    /// The checked addition's block, its overflow flag, and the block returning its sum.
    sum: usize,
    flag: LocalId,
    after: usize,
    /// The written `+`, which every check of the total names.
    span: Span,
}

fn is_plain(types: &TypeTable, local: &LocalDecl) -> bool {
    matches!(types.kind(local.ty), TyKind::Int(_) | TyKind::Uint(_) | TyKind::Bool | TyKind::Void)
}

fn is_marker(stmt: &Stmt) -> bool {
    matches!(stmt.kind, StmtKind::StorageLive(_) | StmtKind::StorageDead(_) | StmtKind::Nop)
}

fn successors(terminator: &Terminator) -> Vec<usize> {
    match terminator {
        Terminator::Goto(next) => vec![next.0 as usize],
        Terminator::SwitchInt { targets, otherwise, .. } => {
            targets.iter().map(|(_, target)| target.0 as usize).chain([otherwise.0 as usize]).collect()
        }
        Terminator::Call { next, .. } | Terminator::Assert { next, .. } => vec![next.0 as usize],
        Terminator::Return | Terminator::Unreachable => Vec::new(),
    }
}

fn retarget(terminator: &mut Terminator, from: usize, to: usize) {
    retarget_all(terminator, &HashMap::from([(from, to)]));
}

/// Each target of `terminator` that `to` maps, to what it maps it to.
fn retarget_all(terminator: &mut Terminator, to: &HashMap<usize, usize>) {
    let each = |target: &mut BasicBlockId| {
        if let Some(&index) = to.get(&(target.0 as usize)) {
            *target = BasicBlockId(index as u32);
        }
    };
    match terminator {
        Terminator::Goto(next) | Terminator::Call { next, .. } | Terminator::Assert { next, .. } => each(next),
        Terminator::SwitchInt { targets, otherwise, .. } => {
            targets.iter_mut().for_each(|(_, target)| each(target));
            each(otherwise);
        }
        Terminator::Return | Terminator::Unreachable => {}
    }
}

/// How many times the body reads `local`.
fn reads(body: &Body, local: LocalId) -> usize {
    let mut count = 0;
    let mut operand = |operand: &Operand| {
        if let Operand::Copy(place) | Operand::Move(place) = operand
            && place.local == local
        {
            count += 1;
        }
    };
    for block in &body.blocks {
        for stmt in &block.stmts {
            match &stmt.kind {
                StmtKind::Assign { rvalue, .. } => rvalue_operands(rvalue).for_each(&mut operand),
                StmtKind::CheckedBinaryOp { lhs, rhs, .. } => {
                    operand(lhs);
                    operand(rhs);
                }
                _ => {}
            }
        }
        match &block.terminator {
            Terminator::SwitchInt { discr, .. } => operand(discr),
            Terminator::Call { args, .. } => args.iter().for_each(&mut operand),
            Terminator::Assert { cond, .. } => operand(cond),
            _ => {}
        }
    }
    count
}

/// The body's shape, when it is the one rewritten and its additions are proved never negative.
fn shape(
    body: &Body,
    types: &TypeTable,
    common: &CommonTypes,
    returns: &HashMap<String, Vec<usize>>,
    fields: &BTreeMap<FieldKey, Interval>,
) -> Option<Shape> {
    let signed = ember_types::is_signed(types, body.locals[0].ty)?;
    if !body.locals.iter().all(|local| is_plain(types, local)) || body.args().any(|(_, local)| ember_types::is_signed(types, local.ty).is_none()) {
        return None;
    }
    // The one checked addition into the result whose block after only returns.
    let mut found = None;
    for (index, block) in body.blocks.iter().enumerate() {
        let Terminator::Assert { cond: Operand::Copy(cond), expected: false, msg: AssertKind::Overflow(BinOp::Add), next, span } = &block.terminator
        else {
            continue;
        };
        let Some((last, before)) = block.stmts.split_last() else { continue };
        let StmtKind::CheckedBinaryOp { dest, overflow, op: BinOp::Add, lhs, rhs } = &last.kind else { continue };
        let after = &body.blocks[next.0 as usize];
        if overflow != cond
            || *dest != Place::local(LocalId(0))
            || !before.iter().all(is_marker)
            || !matches!(after.terminator, Terminator::Return)
            || !after.stmts.iter().all(is_marker)
        {
            continue;
        }
        if found.is_some() {
            return None;
        }
        found = Some((index, cond.local, lhs.clone(), rhs.clone(), *span, next.0 as usize));
    }
    let (sum, flag, lhs, rhs, span, after) = found?;
    // Reached only from a call of the function itself, whose result is one operand and read only there.
    let preds: Vec<usize> = (0..body.blocks.len()).filter(|&block| successors(&body.blocks[block].terminator).contains(&sum)).collect();
    let &[tail] = preds.as_slice() else { return None };
    let Terminator::Call { func: FuncRef::Direct { symbol, latebound: false }, args, dest, .. } = &body.blocks[tail].terminator else {
        return None;
    };
    if *symbol != body.symbol || !dest.projection.is_empty() || args.len() != body.arg_count || reads(body, dest.local) != 1 {
        return None;
    }
    let result = |operand: &Operand| matches!(operand, Operand::Copy(place) | Operand::Move(place) if *place == *dest);
    let added = if result(&lhs) {
        rhs
    } else if result(&rhs) {
        lhs
    } else {
        return None;
    };
    let shape = Shape { tail, args: args.clone(), added, sum, flag, after, span };
    if !signed {
        return Some(shape);
    }
    // The bounds of the parameters: every argument a call of the function itself passes, in the range
    // the facts give it wherever the call is made.
    let any = Analysis::run_seeded(body, types, common, returns, fields, Vec::new())?;
    let calls_of_itself: Vec<usize> = (0..body.blocks.len())
        .filter(|&block| matches!(&body.blocks[block].terminator, Terminator::Call { func: FuncRef::Direct { symbol, .. }, .. } if *symbol == body.symbol))
        .collect();
    let mut bounds: Vec<Option<Interval>> = vec![None; body.arg_count];
    for &block in &calls_of_itself {
        let Terminator::Call { args, .. } = &body.blocks[block].terminator else { unreachable!("a call") };
        for (bound, arg) in bounds.iter_mut().zip(args) {
            match any.range_before(block, body.blocks[block].stmts.len(), arg) {
                Ok(Some(range)) => {
                    *bound = Some(bound.map_or(range, |bound| Interval { lo: bound.lo.min(range.lo), hi: bound.hi.max(range.hi) }));
                }
                Err(()) => {}
                Ok(None) => return None,
            }
        }
    }
    let entry: Vec<(LocalId, Interval)> =
        body.args().map(|(local, _)| local).zip(bounds).map(|(local, bound)| bound.map(|bound| (local, bound))).collect::<Option<_>>()?;
    let within = Analysis::run_seeded(body, types, common, returns, fields, entry)?;
    // Within the bounds, every value returned but the sum is at least 0.
    let result_place = Operand::Copy(Place::local(LocalId(0)));
    for (index, block) in body.blocks.iter().enumerate() {
        if index == shape.after || !matches!(block.terminator, Terminator::Return) {
            continue;
        }
        match within.range_before(index, block.stmts.len(), &result_place) {
            Ok(Some(range)) if range.lo >= 0 => {}
            Err(()) => {}
            _ => return None,
        }
    }
    // The value added: a call of the function itself, at least 0 by this proof; or one the facts bound.
    let by_itself = match &shape.added {
        Operand::Copy(place) | Operand::Move(place) if place.projection.is_empty() => calls_of_itself.iter().any(|&block| {
            matches!(&body.blocks[block].terminator, Terminator::Call { dest, .. } if dest == place)
        }) && reads(body, place.local) == 1,
        _ => false,
    };
    if !by_itself {
        match any.range_before(shape.sum, body.blocks[shape.sum].stmts.len() - 1, &shape.added) {
            Ok(Some(range)) if range.lo >= 0 => {}
            _ => return None,
        }
    }
    Some(shape)
}

fn rewrite(body: &mut Body, shape: Shape, test_at_end: bool) {
    let ty = body.locals[0].ty;
    let span = shape.span;
    let local = |body: &mut Body, ty, name: Option<&str>| {
        body.locals.push(LocalDecl { ty, kind: LocalKind::Temp, name: name.map(str::to_string), span });
        LocalId(body.locals.len() as u32 - 1)
    };
    let bool_ty = body.locals[shape.flag.0 as usize].ty;
    let total = local(body, ty, Some("total"));
    let sum = local(body, ty, None);
    let overflow = local(body, bool_ty, None);
    let params: Vec<(LocalId, LocalId)> = (1..=body.arg_count)
        .map(|index| {
            let param = LocalId(index as u32);
            let ty = body.locals[index].ty;
            (param, local(body, ty, None))
        })
        .collect();
    let assign = |place: LocalId, rvalue: Rvalue| Stmt::new(StmtKind::Assign { place: Place::local(place), rvalue }, span);
    let copy = |local: LocalId| Operand::Copy(Place::local(local));
    let block = |stmts: Vec<Stmt>, terminator: Terminator| BasicBlock { stmts, terminator, terminator_span: span };
    let panics = |unreachable: usize| Terminator::Assert {
        cond: Operand::Const(Const::Bool(true)),
        expected: false,
        msg: AssertKind::Overflow(BinOp::Add),
        next: BasicBlockId(unreachable as u32),
        span,
    };
    // Each turn enters the loop's head again, so no local's storage is begun or ended inside it: the
    // frame holds only numbers and `bool`s.
    for data in &mut body.blocks {
        data.stmts.retain(|stmt| !matches!(stmt.kind, StmtKind::StorageLive(_) | StmtKind::StorageDead(_)));
    }
    // The entry starts the total and goes to the head, the entry as it was.
    let head = body.blocks.len();
    let start = block(vec![assign(total, Rvalue::Use(Operand::Const(Const::Int { value: 0, ty })))], Terminator::Goto(BasicBlockId(head as u32)));
    let entry = std::mem::replace(&mut body.blocks[0], start);
    body.blocks.push(entry);
    for data in &mut body.blocks[1..] {
        retarget(&mut data.terminator, 0, head);
    }
    let moved = |index: usize| if index == 0 { head } else { index };
    // The addition and its return run no more.
    for index in [moved(shape.sum), moved(shape.after)] {
        body.blocks[index] = block(Vec::new(), Terminator::Unreachable);
    }
    let unreachable = body.blocks.len();
    body.blocks.push(block(Vec::new(), Terminator::Unreachable));
    // The turn: the value goes into the total, checked; the call's arguments become the
    // parameters, and the head runs again.
    // For clang and LLVM the turn runs the head itself, so the loop is tested at its end: clang made
    // a loop tested at its top of the C backend's C, and `fib(38)` took 85 ms there against 61 ms
    // (the medians of nine runs on one core); MSVC's is faster tested at its top (65 ms against 80,
    // `ember bench`). A head that is the turn's own block cannot be copied into it.
    let tail = moved(shape.tail);
    let next_turn = body.blocks.len();
    let mut next_stmts = vec![assign(total, Rvalue::Use(copy(sum)))];
    next_stmts.extend(params.iter().map(|(param, temp)| assign(*param, Rvalue::Use(copy(*temp)))));
    let next = if tail == head || !test_at_end {
        Terminator::Goto(BasicBlockId(head as u32))
    } else {
        next_stmts.extend(body.blocks[head].stmts.iter().cloned());
        body.blocks[head].terminator.clone()
    };
    body.blocks.push(block(next_stmts, next));
    // On an overflow, the call the turn stands for is made, as the function would have made it,
    // and then the panic, where the function would have panicked: after its last call.
    let panic = body.blocks.len();
    body.blocks.push(block(Vec::new(), panics(unreachable)));
    let rest = local(body, ty, None);
    let overflowed = body.blocks.len();
    body.blocks.push(block(
        Vec::new(),
        Terminator::Call {
            func: FuncRef::Direct { symbol: body.symbol.clone(), latebound: false },
            args: params.iter().map(|(_, temp)| copy(*temp)).collect(),
            dest: Place::local(rest),
            next: BasicBlockId(panic as u32),
        },
    ));
    let turn = &mut body.blocks[tail];
    for ((_, temp), arg) in params.iter().zip(shape.args) {
        turn.stmts.push(assign(*temp, Rvalue::Use(arg)));
    }
    turn.stmts.push(Stmt::new(
        StmtKind::CheckedBinaryOp { dest: Place::local(sum), overflow: Place::local(overflow), op: BinOp::Add, lhs: copy(total), rhs: shape.added },
        span,
    ));
    turn.terminator = Terminator::SwitchInt {
        discr: copy(overflow),
        targets: vec![(0, BasicBlockId(next_turn as u32))],
        otherwise: BasicBlockId(overflowed as u32),
    };
    // Every return adds what it returns into the total, checked as the written `+` was.
    let returning: Vec<usize> = (0..body.blocks.len()).filter(|&index| matches!(body.blocks[index].terminator, Terminator::Return)).collect();
    for index in returning {
        let back = body.blocks.len();
        body.blocks.push(block(vec![assign(LocalId(0), Rvalue::Use(copy(sum)))], Terminator::Return));
        body.blocks[index].stmts.push(Stmt::new(
            StmtKind::CheckedBinaryOp { dest: Place::local(sum), overflow: Place::local(overflow), op: BinOp::Add, lhs: copy(total), rhs: copy(LocalId(0)) },
            span,
        ));
        body.blocks[index].terminator = Terminator::Assert {
            cond: copy(overflow),
            expected: false,
            msg: AssertKind::Overflow(BinOp::Add),
            next: BasicBlockId(back as u32),
            span,
        };
    }
}
