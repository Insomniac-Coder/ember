//! Room for a counted loop's pushes, asked for before it.
//!
//! A list filled by a loop (`for i in 0..n: xs.push(..)`) grows by doubling:
//! about twenty copies of what it holds on the way to a million elements, and
//! pages touched twice. C written by hand allocates the million at once (the
//! enum `match` benchmark's fill: 1.12x C on clang, and C's speed with the
//! room asked for first). When a counted loop with no early exit pushes onto
//! the same list at least `k` times on every path through a turn, the room for
//! `k` times its turns is asked for before it: `ArrayReserveHint`, which never
//! fails (with no memory, or a count past the limit, the list grows as it
//! would have) and changes nothing but the capacity. So it is done only in a
//! program that never asks a list for its `capacity()`; the memory it takes
//! is no more than doubling would have reached (`[PHIL-5]`, `[PHIL-11]`).

use std::collections::{BTreeMap, HashSet};

use ember_mir::{BasicBlock, BasicBlockId, Body, Builtin, FuncRef, LocalId, Operand, Place, Rvalue, StmtKind, Terminator};
use ember_types::TypeTable;

use crate::loop_version::{CountedLoop, counted_loop, retarget};

pub fn reserve_pushed_lists_all(bodies: &mut [Body], types: &TypeTable) -> usize {
    if bodies.iter().any(reads_capacity) {
        return 0;
    }
    bodies.iter_mut().map(|body| reserve_pushed_lists(body, types)).sum()
}

fn reads_capacity(body: &Body) -> bool {
    body.blocks.iter().any(|data| {
        matches!(&data.terminator, Terminator::Call { func: FuncRef::Builtin { which: Builtin::ArrayCapacity, .. }, .. })
    })
}

fn reserve_pushed_lists(body: &mut Body, types: &TypeTable) -> usize {
    // A hint adds blocks at the end and keeps every block's number, so each
    // header is looked at once.
    let headers = body.blocks.len();
    let mut count = 0;
    for header in 0..headers {
        if let Some(shape) = counted_loop(body, types, header) {
            count += reserve_before(body, types, &shape);
        }
    }
    count
}

fn successors(terminator: &Terminator) -> Vec<usize> {
    match terminator {
        Terminator::Goto(next) | Terminator::Call { next, .. } | Terminator::Assert { next, .. } => vec![next.0 as usize],
        Terminator::SwitchInt { targets, otherwise, .. } => {
            targets.iter().map(|(_, target)| target.0 as usize).chain([otherwise.0 as usize]).collect()
        }
        Terminator::Return | Terminator::Unreachable => Vec::new(),
    }
}

/// Built-ins that take elements away: a loop using one may not keep what
/// it pushes, and room asked for at the start would be memory held for
/// nothing.
fn shrinks(which: &Builtin) -> bool {
    matches!(
        which,
        Builtin::ArrayPop { .. }
            | Builtin::ArrayClear
            | Builtin::ArrayRemove
            | Builtin::ArrayDrain
            | Builtin::ArrayTruncate
            | Builtin::ArraySwapRemove
    )
}

/// The turns a loop takes when its counter starts at a constant (the one
/// value it is given before the loop) and its limit is one.
fn constant_turns(body: &Body, types: &TypeTable, shape: &CountedLoop, inside: &HashSet<usize>) -> Option<i128> {
    let value_of = |local: LocalId, outside_only: bool| -> Option<i128> {
        let mut found = None;
        for (block, data) in body.blocks.iter().enumerate() {
            if outside_only && inside.contains(&block) {
                continue;
            }
            for stmt in &data.stmts {
                if let StmtKind::Assign { place, rvalue } = &stmt.kind
                    && place.local == local
                {
                    let Rvalue::Use(Operand::Const(ember_mir::Const::Int { value, ty })) = rvalue else { return None };
                    if found.is_some() || !place.projection.is_empty() {
                        return None;
                    }
                    found = Some(crate::range_facts::constant(types, *value, *ty)?);
                }
            }
        }
        found
    };
    let start = value_of(shape.counter, true)?;
    let limit = value_of(shape.limit, false)?;
    Some((limit - start + i128::from(shape.inclusive)).max(0))
}

fn reserve_before(body: &mut Body, types: &TypeTable, shape: &CountedLoop) -> usize {
    let region: HashSet<usize> = shape.region.iter().copied().collect();
    let mut inside = region.clone();
    inside.insert(shape.header);
    // Every turn runs to the step: nothing leaves but the header's test.
    for &block in &region {
        let terminator = &body.blocks[block].terminator;
        if matches!(terminator, Terminator::Return)
            || successors(terminator).iter().any(|next| !inside.contains(next))
        {
            return 0;
        }
    }
    // The counter moves only in the step and the limit not at all, so the
    // turns are `limit - counter` as the loop starts.
    let taken: HashSet<LocalId> = body
        .blocks
        .iter()
        .flat_map(|data| &data.stmts)
        .filter_map(|stmt| match &stmt.kind {
            StmtKind::Assign { rvalue: Rvalue::Ref { place, .. }, .. } => Some(place.local),
            _ => None,
        })
        .collect();
    if taken.contains(&shape.counter) || taken.contains(&shape.limit) {
        return 0;
    }
    let mut counter_writes = 0;
    for &block in &inside {
        let data = &body.blocks[block];
        for stmt in &data.stmts {
            let written: Vec<LocalId> = match &stmt.kind {
                StmtKind::Assign { place, .. } | StmtKind::Drop { place, .. } => vec![place.local],
                StmtKind::CheckedBinaryOp { dest, overflow, .. } => vec![dest.local, overflow.local],
                _ => Vec::new(),
            };
            if written.contains(&shape.limit) {
                return 0;
            }
            counter_writes += written.iter().filter(|&&local| local == shape.counter).count();
        }
        if let Terminator::Call { dest, func, .. } = &data.terminator {
            if dest.local == shape.counter || dest.local == shape.limit {
                return 0;
            }
            if let FuncRef::Builtin { which, .. } = func
                && shrinks(which)
            {
                return 0;
            }
        }
    }
    if counter_writes != 1 {
        return 0;
    }

    // The lists pushed onto: the local each push's reference was made from,
    // `r = &mut xs` (a reference made anywhere, for one push or for all).
    let mut made_from: BTreeMap<u32, Vec<&Rvalue>> = BTreeMap::new();
    let mut given: BTreeMap<u32, usize> = BTreeMap::new();
    for data in &body.blocks {
        for stmt in &data.stmts {
            match &stmt.kind {
                StmtKind::Assign { place, rvalue } => {
                    *given.entry(place.local.0).or_default() += 1;
                    if place.projection.is_empty() {
                        made_from.entry(place.local.0).or_default().push(rvalue);
                    }
                }
                StmtKind::CheckedBinaryOp { dest, overflow, .. } => {
                    *given.entry(dest.local.0).or_default() += 1;
                    *given.entry(overflow.local.0).or_default() += 1;
                }
                _ => {}
            }
        }
        if let Terminator::Call { dest, .. } = &data.terminator {
            *given.entry(dest.local.0).or_default() += 1;
        }
    }
    // Every value the reference is given is `&xs` of the one local: loop
    // versioning (`[OPT-2]`) copies a loop's blocks, and with them the
    // statement making it.
    let list_of = |reference: LocalId| -> Option<LocalId> {
        let made = made_from.get(&reference.0)?;
        if given.get(&reference.0) != Some(&made.len()) {
            return None;
        }
        let mut list = None;
        for rvalue in made {
            let Rvalue::Ref { place, .. } = rvalue else { return None };
            if !place.projection.is_empty() || list.is_some_and(|seen| seen != place.local) {
                return None;
            }
            list = Some(place.local);
        }
        list
    };
    // block -> (reference, list); list -> (the push's `arg_ty`, the reference's type)
    let mut pushes: BTreeMap<usize, (LocalId, LocalId)> = BTreeMap::new();
    let mut lists: BTreeMap<u32, (ember_types::Ty, ember_types::Ty)> = BTreeMap::new();
    for &block in &region {
        if let Terminator::Call { func: FuncRef::Builtin { which: Builtin::ArrayPush, arg_ty }, args, .. } = &body.blocks[block].terminator
            && let Some(Operand::Copy(place) | Operand::Move(place)) = args.first()
            && place.projection.is_empty()
            && let Some(list) = list_of(place.local)
        {
            pushes.insert(block, (place.local, list));
            lists.entry(list.0).or_insert((*arg_ty, body.local(place.local).ty));
        }
    }
    let order = forward_order(body, shape);
    let turns = constant_turns(body, types, shape, &inside);
    let mut hints: Vec<(LocalId, ember_types::Ty, ember_types::Ty, u32)> = Vec::new();
    for (list, (arg_ty, reference_ty)) in lists {
        let list = LocalId(list);
        let references: HashSet<LocalId> =
            pushes.values().filter(|(_, onto)| *onto == list).map(|(reference, _)| *reference).collect();
        if !only_pushes(body, &inside, list, &references) {
            continue;
        }
        let per_turn = fewest_pushes(shape, &order, &pushes, list);
        // All the pushes fit in the first growth (four slots): nothing to
        // save, and no code to add.
        if turns.is_some_and(|turns| turns * i128::from(per_turn) <= 4) {
            continue;
        }
        if per_turn > 0 {
            hints.push((list, arg_ty, reference_ty, per_turn));
        }
    }
    if hints.is_empty() {
        return 0;
    }

    // Before the loop: one block per list, each making its own reference to
    // the list, the last going to the header.
    let span = body.blocks[shape.header].terminator_span;
    let void = body.blocks.iter().find_map(|data| match &data.terminator {
        Terminator::Call { func: FuncRef::Builtin { which: Builtin::ArrayPush, .. }, dest, .. } => Some(body.local(dest.local).ty),
        _ => None,
    });
    let Some(void) = void else { return 0 };
    let first = body.blocks.len();
    let header = BasicBlockId(shape.header as u32);
    let count = hints.len();
    for (index, (list, arg_ty, reference_ty, per_turn)) in hints.into_iter().enumerate() {
        body.locals.push(ember_mir::LocalDecl { ty: reference_ty, kind: ember_mir::LocalKind::Temp, name: None, span });
        let reference = LocalId(body.locals.len() as u32 - 1);
        body.locals.push(ember_mir::LocalDecl { ty: void, kind: ember_mir::LocalKind::Temp, name: None, span });
        let dest = LocalId(body.locals.len() as u32 - 1);
        let next = if index + 1 == count { header } else { BasicBlockId((first + index + 1) as u32) };
        body.blocks.push(BasicBlock {
            stmts: vec![ember_mir::Stmt::new(
                StmtKind::Assign { place: Place::local(reference), rvalue: Rvalue::Ref { place: Place::local(list), mutable: true } },
                span,
            )],
            terminator: Terminator::Call {
                func: FuncRef::Builtin {
                    which: Builtin::ArrayReserveHint { per_turn, inclusive: shape.inclusive },
                    arg_ty,
                },
                args: vec![
                    Operand::Copy(Place::local(reference)),
                    Operand::Copy(Place::local(shape.counter)),
                    Operand::Copy(Place::local(shape.limit)),
                ],
                dest: Place::local(dest),
                next,
            },
            terminator_span: span,
        });
    }
    let preheader = BasicBlockId(first as u32);
    for block in 0..first {
        if !inside.contains(&block) {
            retarget(&mut body.blocks[block].terminator, |target| if target == header { preheader } else { target });
        }
    }
    count
}

/// Whether, in the loop, the list is changed only by its pushes: it is not
/// assigned, dropped or moved, every mutable reference to it made there is
/// one of the `references` the pushes use, and those are used for nothing
/// else.
fn only_pushes(body: &Body, inside: &HashSet<usize>, list: LocalId, references: &HashSet<LocalId>) -> bool {
    let moves_list = |operand: &Operand| matches!(operand, Operand::Move(place) if place.local == list);
    for &block in inside {
        let data = &body.blocks[block];
        for stmt in &data.stmts {
            match &stmt.kind {
                StmtKind::Assign { place, rvalue } => {
                    if place.local == list || (references.contains(&place.local) && !matches!(rvalue, Rvalue::Ref { .. })) {
                        return false;
                    }
                    let moved = match rvalue {
                        Rvalue::Use(value) | Rvalue::Cast { operand: value, .. } | Rvalue::UnaryOp { operand: value, .. } => moves_list(value),
                        Rvalue::BinaryOp { lhs, rhs, .. } => moves_list(lhs) || moves_list(rhs),
                        Rvalue::Aggregate { operands, .. } => operands.iter().any(moves_list),
                        Rvalue::Repeat { value, .. } => moves_list(value),
                        Rvalue::Ref { place: target, mutable } => {
                            *mutable && target.local == list && !references.contains(&place.local)
                        }
                        Rvalue::Discriminant(_) => false,
                    };
                    if moved {
                        return false;
                    }
                }
                StmtKind::Drop { place, .. } if place.local == list => return false,
                StmtKind::CheckedBinaryOp { dest, overflow, .. } if dest.local == list || overflow.local == list => return false,
                _ => {}
            }
            let mut read = vec![false; body.locals.len()];
            crate::strength_reduce::stmt_reads(stmt, &mut read);
            if references.iter().any(|reference| read[reference.0 as usize]) {
                return false;
            }
        }
        let mut read = vec![false; body.locals.len()];
        match &data.terminator {
            Terminator::Call { func: FuncRef::Builtin { which: Builtin::ArrayPush, .. }, args, dest, .. }
                if matches!(args.first(), Some(Operand::Copy(place) | Operand::Move(place)) if references.contains(&place.local)) =>
            {
                args[1..].iter().for_each(|arg| crate::strength_reduce::operand_reads(arg, &mut read));
                if dest.local == list {
                    return false;
                }
            }
            terminator => {
                crate::strength_reduce::terminator_reads(terminator, &mut read);
                if let Terminator::Call { args, dest, .. } = terminator
                    && (dest.local == list || args.iter().any(moves_list))
                {
                    return false;
                }
            }
        }
        if references.iter().any(|reference| read[reference.0 as usize]) {
            return false;
        }
    }
    true
}

/// The loop's body in an order that puts each block after every block it
/// goes to, but for the edges back to the header or into an inner loop's
/// header (a depth-first walk's back edges).
fn forward_order(body: &Body, shape: &CountedLoop) -> Vec<(usize, Vec<usize>)> {
    let region: HashSet<usize> = shape.region.iter().copied().collect();
    let mut post = Vec::new();
    let mut state: BTreeMap<usize, bool> = BTreeMap::new(); // false: open, true: done
    let mut stack: Vec<(usize, Vec<usize>, usize)> = Vec::new();
    let forward = |block: usize| -> Vec<usize> {
        successors(&body.blocks[block].terminator).into_iter().filter(|next| region.contains(next)).collect()
    };
    stack.push((shape.entry, forward(shape.entry), 0));
    state.insert(shape.entry, false);
    let mut kept: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    while let Some((block, nexts, at)) = stack.pop() {
        if at < nexts.len() {
            let next = nexts[at];
            stack.push((block, nexts.clone(), at + 1));
            match state.get(&next) {
                None => {
                    kept.entry(block).or_default().push(next);
                    state.insert(next, false);
                    stack.push((next, forward(next), 0));
                }
                // Done: a forward or cross edge. Open: a back edge, dropped.
                Some(true) => kept.entry(block).or_default().push(next),
                Some(false) => {}
            }
        } else {
            state.insert(block, true);
            post.push((block, kept.remove(&block).unwrap_or_default()));
        }
    }
    post
}

/// The fewest pushes onto `list` on any path from the loop's entry to its
/// step (zero when a path has none).
fn fewest_pushes(
    shape: &CountedLoop,
    order: &[(usize, Vec<usize>)],
    pushes: &BTreeMap<usize, (LocalId, LocalId)>,
    list: LocalId,
) -> u32 {
    let mut fewest: BTreeMap<usize, u32> = BTreeMap::new();
    for (block, nexts) in order {
        let own = u32::from(pushes.get(block).is_some_and(|(_, onto)| *onto == list));
        let after = if *block == shape.step {
            Some(0)
        } else {
            nexts.iter().filter_map(|next| fewest.get(next).copied()).min()
        };
        // A block whose only ways on go back into an inner loop reaches the
        // step through that loop's header, already counted there.
        if let Some(after) = after {
            fewest.insert(*block, own.saturating_add(after));
        }
    }
    fewest.get(&shape.entry).copied().unwrap_or(0)
}
