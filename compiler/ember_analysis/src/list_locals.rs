//! A list a loop only pushes onto lives in a local for the loop.
//!
//! A list's header (its buffer, length and capacity) and its buffer both come
//! from the runtime, which the C compiler cannot see into, so it cannot tell
//! that writing an element never changes the header: after every element a
//! loop pushes it reads the length (and the buffer) from memory again, and a
//! loop of pushes stays one element at a time. C written by hand gets its
//! buffer from `realloc`, which the C compiler knows is memory of its own, and
//! keeps the length in a register. Ember knows a list owns its buffer, so a
//! list that an innermost loop changes only by pushing is read into a local
//! before the loop and written back on every way out of it: the C compiler
//! then keeps all three in registers (the push's growth takes and gives back
//! the list by value, so the local's address never escapes). Measured on the
//! view-held benchmark (pushing a list's thousand numbers onto another list
//! held in an object, a hundred thousand times) with gcc: 1.03x the
//! hand-written C before, 0.42x after.
//!
//! The list is a local (`xs`) or a field path from one (`b.log`, through a
//! class handle, a struct, or a reference to either). Nothing else in the loop
//! may reach it: no call but the built-in operations, none of them given
//! anything that could hold or point at the list, and no other use of the list
//! or of what holds it (another handle of any class could be the same object).
//! A panic in the loop leaves the list in memory as it was when the loop
//! began, which nothing sees: a panic ends the program (`[PAN-1]`).

use std::collections::{BTreeMap, BTreeSet, HashSet};

use ember_mir::{
    AssertKind, BasicBlock, BasicBlockId, Body, Builtin, FuncRef, LocalDecl, LocalId, LocalKind, Operand, Place, Projection,
    Rvalue, Stmt, StmtKind, Terminator,
};
use ember_types::{Ty, TyKind, TypeTable};

use crate::loop_version::retarget;
use crate::range_facts::{loop_headers, natural_loops, reverse_postorder};
use crate::regions::place_type;

pub fn keep_pushed_lists_in_locals_all(bodies: &mut [Body], types: &TypeTable) -> usize {
    bodies.iter_mut().map(|body| keep_pushed_lists_in_locals(body, types)).sum()
}

fn keep_pushed_lists_in_locals(body: &mut Body, types: &TypeTable) -> usize {
    let order = reverse_postorder(body);
    let headers = loop_headers(body, &order);
    // Innermost loops: they hold no other loop's header. They are disjoint, so
    // the blocks one adds and the edges it moves leave the others' as found.
    let innermost: BTreeMap<usize, HashSet<usize>> = natural_loops(body, &order, &headers)
        .into_iter()
        .filter(|(header, inside)| inside.iter().all(|block| block == header || !headers.contains(block)))
        .collect();
    innermost.iter().map(|(&header, inside)| keep_in_locals(body, types, header, inside)).sum()
}

/// Every value each local is given anywhere: the place a reference made inside
/// the loop points at, or `None` for anything else.
fn given(body: &Body, inside: &HashSet<usize>) -> BTreeMap<LocalId, Vec<Option<Place>>> {
    let mut made: BTreeMap<LocalId, Vec<Option<Place>>> = BTreeMap::new();
    for (block, data) in body.blocks.iter().enumerate() {
        for stmt in &data.stmts {
            match &stmt.kind {
                StmtKind::Assign { place, rvalue } => {
                    let target = match rvalue {
                        Rvalue::Ref { place: target, .. } if inside.contains(&block) && place.projection.is_empty() => Some(target.clone()),
                        _ => None,
                    };
                    made.entry(place.local).or_default().push(target);
                }
                StmtKind::CheckedBinaryOp { dest, overflow, .. } => {
                    made.entry(dest.local).or_default().push(None);
                    made.entry(overflow.local).or_default().push(None);
                }
                _ => {}
            }
        }
        if let Terminator::Call { dest, .. } = &data.terminator {
            made.entry(dest.local).or_default().push(None);
        }
    }
    made
}

/// The lists the loop's pushes are given, each with its receivers: the
/// references `r = &mut list` the pushes take, every value the loop gives `r`
/// being that. Loop versioning (`[OPT-2]`) copies a loop with the statements
/// making its references, so a receiver may also be given values outside.
fn pushed_lists(body: &Body, inside: &HashSet<usize>) -> BTreeMap<Place, HashSet<LocalId>> {
    let mut made_inside: BTreeMap<LocalId, Vec<Option<Place>>> = BTreeMap::new();
    for &block in inside {
        let data = &body.blocks[block];
        for stmt in &data.stmts {
            if let StmtKind::Assign { place, rvalue } = &stmt.kind {
                let list = match rvalue {
                    Rvalue::Ref { place: list, mutable: true } if place.projection.is_empty() => Some(list.clone()),
                    _ => None,
                };
                made_inside.entry(place.local).or_default().push(list);
            }
            if let StmtKind::CheckedBinaryOp { dest, overflow, .. } = &stmt.kind {
                made_inside.entry(dest.local).or_default().push(None);
                made_inside.entry(overflow.local).or_default().push(None);
            }
        }
        if let Terminator::Call { dest, .. } = &data.terminator {
            made_inside.entry(dest.local).or_default().push(None);
        }
    }
    let mut lists: BTreeMap<Place, HashSet<LocalId>> = BTreeMap::new();
    for &block in inside {
        let Terminator::Call { func: FuncRef::Builtin { which: Builtin::ArrayPush, .. }, args, .. } = &body.blocks[block].terminator
        else {
            continue;
        };
        let Some(Operand::Copy(receiver) | Operand::Move(receiver)) = args.first() else { continue };
        if !receiver.projection.is_empty() {
            continue;
        }
        let Some(values) = made_inside.get(&receiver.local) else { continue };
        let Some(Some(list)) = values.first() else { continue };
        if values.iter().all(|value| value.as_ref() == Some(list)) {
            lists.entry(list.clone()).or_default().insert(receiver.local);
        }
    }
    lists
}

/// Per block, whether `local`'s value on entry may still be read.
fn live_on_entry(body: &Body, local: LocalId) -> Vec<bool> {
    let count = body.blocks.len();
    // Per block: read before any write, and written at all.
    let mut uses = vec![false; count];
    let mut defines = vec![false; count];
    for (block, data) in body.blocks.iter().enumerate() {
        let mut written = false;
        for stmt in &data.stmts {
            let mut read = vec![false; body.locals.len()];
            crate::strength_reduce::stmt_reads(stmt, &mut read);
            if read[local.0 as usize] && !written {
                uses[block] = true;
            }
            if let StmtKind::Assign { place, .. } = &stmt.kind
                && place.local == local
                && place.projection.is_empty()
            {
                written = true;
            }
        }
        let mut read = vec![false; body.locals.len()];
        for operand in terminator_operands(&data.terminator) {
            crate::strength_reduce::operand_reads(operand, &mut read);
        }
        if read[local.0 as usize] && !written {
            uses[block] = true;
        }
        if let Terminator::Call { dest, .. } = &data.terminator
            && dest.local == local
        {
            written = true;
        }
        defines[block] = written;
    }
    let mut live = uses.clone();
    loop {
        let mut changed = false;
        for block in (0..count).rev() {
            if live[block] || defines[block] {
                continue;
            }
            if crate::long_access_lint::successors(&body.blocks[block].terminator).into_iter().any(|next| live[next]) {
                live[block] = true;
                changed = true;
            }
        }
        if !changed {
            return live;
        }
    }
}

fn keep_in_locals(body: &mut Body, types: &TypeTable, header: usize, inside: &HashSet<usize>) -> usize {
    // A way out of the loop that is not an edge (a `return` inside it) would
    // leave the list unwritten; the entry block has no edge to put the read on.
    if header == 0 || inside.iter().any(|&block| matches!(body.blocks[block].terminator, Terminator::Return)) {
        return 0;
    }
    let targets: BTreeSet<usize> = inside
        .iter()
        .flat_map(|&block| crate::long_access_lint::successors(&body.blocks[block].terminator))
        .filter(|target| !inside.contains(target))
        .collect();
    let made = given(body, inside);
    let kept: Vec<(Place, HashSet<LocalId>)> = pushed_lists(body, inside)
        .into_iter()
        .filter(|(list, receivers)| {
            in_place(body, types, list)
                && matches!(types.kind(place_type(body, types, list)), TyKind::Vec { .. })
                // Each receiver's value in the loop is the loop's own: nothing
                // made before the loop is read in it, nothing it makes is read
                // after it.
                && receivers.iter().all(|&receiver| {
                    let live = live_on_entry(body, receiver);
                    !live[header] && targets.iter().all(|&target| !live[target])
                })
                && only_pushed(body, types, inside, &made, list, receivers)
        })
        .collect();
    if kept.is_empty() {
        return 0;
    }
    let span = body.blocks[header].terminator_span;
    let mut reads = Vec::new();
    let mut writes = Vec::new();
    for (list, receivers) in &kept {
        let ty = place_type(body, types, list);
        body.locals.push(LocalDecl { ty, kind: LocalKind::Temp, name: None, span });
        let local = Place::local(LocalId(body.locals.len() as u32 - 1));
        for &block in inside {
            for stmt in &mut body.blocks[block].stmts {
                if let StmtKind::Assign { place, rvalue: Rvalue::Ref { place: target, .. } } = &mut stmt.kind
                    && receivers.contains(&place.local)
                {
                    *target = local.clone();
                }
            }
        }
        reads.push(Stmt::new(StmtKind::Assign { place: local.clone(), rvalue: Rvalue::Use(Operand::Copy(list.clone())) }, span));
        writes.push(Stmt::new(StmtKind::Assign { place: list.clone(), rvalue: Rvalue::Use(Operand::Copy(local)) }, span));
    }

    // Read before the loop: every edge into the header from outside it goes
    // through one block that reads.
    let preheader = BasicBlockId(body.blocks.len() as u32);
    let header_id = BasicBlockId(header as u32);
    for block in 0..body.blocks.len() {
        if !inside.contains(&block) {
            retarget(&mut body.blocks[block].terminator, |target| if target == header_id { preheader } else { target });
        }
    }
    body.blocks.push(BasicBlock { stmts: reads, terminator: Terminator::Goto(header_id), terminator_span: span });

    // Written back on every edge out: one block per place the loop leaves to,
    // numbered in the order they are made.
    let exits: BTreeMap<u32, BasicBlockId> = targets
        .iter()
        .enumerate()
        .map(|(index, &target)| (target as u32, BasicBlockId((body.blocks.len() + index) as u32)))
        .collect();
    for &block in inside {
        retarget(&mut body.blocks[block].terminator, |target| exits.get(&target.0).copied().unwrap_or(target));
    }
    for &target in exits.keys() {
        body.blocks.push(BasicBlock {
            stmts: writes.clone(),
            terminator: Terminator::Goto(BasicBlockId(target)),
            terminator_span: span,
        });
    }
    kept.len()
}

/// Whether the list is a local, or a field path from one, possibly through
/// the reference the local is: somewhere the loop cannot move.
fn in_place(body: &Body, types: &TypeTable, list: &Place) -> bool {
    list.projection.iter().enumerate().all(|(depth, step)| match step {
        Projection::Field(_) => true,
        Projection::Deref => depth == 0 && matches!(types.kind(body.local(list.local).ty), TyKind::Ref { .. }),
        _ => false,
    })
}

/// Whether, in the loop, the list is reached only by its pushes: through the
/// `receivers`, made from the list and given to `push` and nothing else.
fn only_pushed(
    body: &Body,
    types: &TypeTable,
    inside: &HashSet<usize>,
    made: &BTreeMap<LocalId, Vec<Option<Place>>>,
    list: &Place,
    receivers: &HashSet<LocalId>,
) -> bool {
    let list_ty = place_type(body, types, list);
    // In an object when any step of the path to it is a class handle; else in
    // the storage of the local, or of what the local refers to.
    let behind_handle = (0..list.projection.len()).any(|depth| {
        let prefix = Place { local: list.local, projection: list.projection[..depth].to_vec() };
        matches!(types.kind(place_type(body, types, &prefix)), TyKind::Class(_))
    });
    let reaches_raw = |place: &Place| -> bool {
        if place.local == list.local {
            // The same local: only a field path apart from the list's leaves it.
            let apart = place
                .projection
                .iter()
                .zip(&list.projection)
                .find(|(a, b)| a != b)
                .is_some_and(|(a, b)| matches!((a, b), (Projection::Field(_), Projection::Field(_))));
            return !apart;
        }
        // Another local reaches the list through a reference or pointer to
        // where it lives, and, in an object, through any handle (any could be
        // the list's object).
        let ty = body.local(place.local).ty;
        points_at(types, ty, list_ty, &mut HashSet::new()) || (behind_handle && refers_to_object(types, ty, &mut HashSet::new()))
    };
    // Temporaries the loop makes as references, every value they are given
    // being one made there to something that does not reach the list: what
    // they point at is known, and is not the list.
    let known: HashSet<LocalId> = made
        .iter()
        .filter(|(local, targets)| {
            !receivers.contains(local)
                && body.local(**local).kind == LocalKind::Temp
                && targets.iter().all(|target| target.as_ref().is_some_and(|target| !reaches_raw(target)))
        })
        .map(|(local, _)| *local)
        .collect();
    let reaches = |place: &Place| receivers.contains(&place.local) || (!known.contains(&place.local) && reaches_raw(place));
    let operand_reaches = |operand: &Operand| matches!(operand, Operand::Copy(place) | Operand::Move(place) if reaches(place));
    for &block in inside {
        let data = &body.blocks[block];
        for stmt in &data.stmts {
            let clear = match &stmt.kind {
                StmtKind::Assign { place, rvalue } if receivers.contains(&place.local) => {
                    place.projection.is_empty() && matches!(rvalue, Rvalue::Ref { place: target, mutable: true } if target == list)
                }
                StmtKind::Assign { place, rvalue: Rvalue::Ref { place: target, .. } } if known.contains(&place.local) => {
                    place.projection.is_empty() && !reaches_raw(target)
                }
                StmtKind::Assign { place, rvalue } => {
                    !reaches(place)
                        && match rvalue {
                            Rvalue::Use(value) | Rvalue::Cast { operand: value, .. } | Rvalue::UnaryOp { operand: value, .. } => {
                                !operand_reaches(value)
                            }
                            Rvalue::BinaryOp { lhs, rhs, .. } => !operand_reaches(lhs) && !operand_reaches(rhs),
                            Rvalue::Aggregate { operands, .. } => !operands.iter().any(operand_reaches),
                            Rvalue::Repeat { value, .. } => !operand_reaches(value),
                            Rvalue::Discriminant(target) | Rvalue::Ref { place: target, .. } => !reaches(target),
                        }
                }
                // An access check touches the object's access count, never the list.
                StmtKind::BeginAccess { .. }
                | StmtKind::EndAccess { .. }
                | StmtKind::BeginAccessTransfer { .. }
                | StmtKind::EndAccessTransfer { .. } => true,
                StmtKind::CheckedBinaryOp { dest, overflow, lhs, rhs, .. } => {
                    !reaches(dest) && !reaches(overflow) && !operand_reaches(lhs) && !operand_reaches(rhs)
                }
                StmtKind::StorageLive(local) | StmtKind::StorageDead(local) => *local != list.local,
                StmtKind::Drop { place, .. } => !reaches(place),
                StmtKind::Nop => true,
            };
            if !clear {
                return false;
            }
        }
        let clear = match &data.terminator {
            Terminator::Call { func: FuncRef::Builtin { which: Builtin::ArrayPush, .. }, args, dest, .. }
                if matches!(args.first(), Some(Operand::Copy(place) | Operand::Move(place)) if receivers.contains(&place.local)) =>
            {
                !args[1..].iter().any(operand_reaches) && !reaches(dest)
            }
            Terminator::Call { func: FuncRef::Builtin { .. }, args, dest, .. } => !args.iter().any(operand_reaches) && !reaches(dest),
            Terminator::Call { .. } | Terminator::Return => false,
            terminator => !terminator_operands(terminator).into_iter().any(operand_reaches),
        };
        if !clear {
            return false;
        }
    }
    true
}

fn terminator_operands(terminator: &Terminator) -> Vec<&Operand> {
    match terminator {
        Terminator::SwitchInt { discr, .. } => vec![discr],
        Terminator::Call { args, .. } => args.iter().collect(),
        Terminator::Assert { cond, msg, .. } => {
            let mut operands = vec![cond];
            match msg {
                AssertKind::Bounds { len, index } => operands.extend([len, index]),
                AssertKind::RefCellBorrow { file, line } => operands.extend([file, line]),
                AssertKind::Panic { message } => operands.push(message),
                _ => {}
            }
            operands
        }
        Terminator::Goto(_) | Terminator::Return | Terminator::Unreachable => Vec::new(),
    }
}

/// Whether a value of `ty` holds a class handle, or anything that could be
/// one, so it could reach a field of some object.
fn refers_to_object(types: &TypeTable, ty: Ty, seen: &mut HashSet<Ty>) -> bool {
    if !seen.insert(ty) {
        return false;
    }
    match types.kind(ty).clone() {
        TyKind::Class(_)
        | TyKind::ClassInterface(_)
        | TyKind::Dyn { .. }
        | TyKind::Opaque(_)
        | TyKind::Param { .. }
        | TyKind::Assoc { .. }
        | TyKind::Infer(_)
        | TyKind::Fn { .. } => true,
        TyKind::Struct(id) => types.struct_def(id).fields.iter().any(|field| refers_to_object(types, field.ty, seen)),
        TyKind::Enum(id) => types
            .enum_def(id)
            .variants
            .iter()
            .any(|variant| variant.fields.iter().any(|field| refers_to_object(types, field.ty, seen))),
        TyKind::Tuple(items) => items.iter().any(|&item| refers_to_object(types, item, seen)),
        TyKind::Vec { elem, .. } | TyKind::Array { elem, .. } | TyKind::Span { elem, .. } => refers_to_object(types, elem, seen),
        TyKind::Ref { inner, .. } | TyKind::Ptr { inner, .. } => refers_to_object(types, inner, seen),
        _ => false,
    }
}

/// Whether a value of `ty` holds a reference or pointer (or anything that
/// could be one) to storage holding a `list_ty` by value: a local's list is
/// reached from elsewhere only so. A view (`Span`) shows a buffer's elements,
/// never a local's storage.
fn points_at(types: &TypeTable, ty: Ty, list_ty: Ty, seen: &mut HashSet<Ty>) -> bool {
    if !seen.insert(ty) {
        return false;
    }
    match types.kind(ty).clone() {
        TyKind::Ref { inner, .. } | TyKind::Ptr { inner, .. } => {
            holds(types, inner, list_ty, &mut HashSet::new()) || points_at(types, inner, list_ty, seen)
        }
        TyKind::Dyn { .. } | TyKind::Opaque(_) | TyKind::Param { .. } | TyKind::Assoc { .. } | TyKind::Infer(_) | TyKind::Fn { .. } => true,
        TyKind::Struct(id) => types.struct_def(id).fields.iter().any(|field| points_at(types, field.ty, list_ty, seen)),
        TyKind::Enum(id) => types
            .enum_def(id)
            .variants
            .iter()
            .any(|variant| variant.fields.iter().any(|field| points_at(types, field.ty, list_ty, seen))),
        TyKind::Tuple(items) => items.iter().any(|&item| points_at(types, item, list_ty, seen)),
        TyKind::Vec { elem, .. } | TyKind::Array { elem, .. } | TyKind::Span { elem, .. } => points_at(types, elem, list_ty, seen),
        _ => false,
    }
}

/// Whether storage of type `ty` holds a `list_ty` by value.
fn holds(types: &TypeTable, ty: Ty, list_ty: Ty, seen: &mut HashSet<Ty>) -> bool {
    if ty == list_ty {
        return true;
    }
    if !seen.insert(ty) {
        return false;
    }
    match types.kind(ty).clone() {
        TyKind::Struct(id) => types.struct_def(id).fields.iter().any(|field| holds(types, field.ty, list_ty, seen)),
        TyKind::Enum(id) => {
            types.enum_def(id).variants.iter().any(|variant| variant.fields.iter().any(|field| holds(types, field.ty, list_ty, seen)))
        }
        TyKind::Tuple(items) => items.iter().any(|&item| holds(types, item, list_ty, seen)),
        TyKind::Array { elem, .. } => holds(types, elem, list_ty, seen),
        TyKind::Dyn { .. } | TyKind::Opaque(_) | TyKind::Param { .. } | TyKind::Assoc { .. } | TyKind::Infer(_) => true,
        _ => false,
    }
}
