//! Conservative static exclusivity elision (`[EXC-3]`).
//!
//! This pass intentionally proves only the smallest useful `unique_handle`
//! case: an access rooted at an unprojected local counted owner that is never
//! copied, passed, returned, or otherwise escaped. `Shared[T].get_mut()` uses
//! one compiler-created `ref mut Shared[T]` temporary; that temporary can be
//! traced back to its one local owner. Anything involving an indexed/projection
//! root, or an unknown handle use remains dynamic and therefore fail-closed.
//!
//! A second pass (`[EXC-3]`'s whole-program case) removes the checks that
//! nothing held anywhere in the program could make fail.

use std::collections::HashSet;

use ember_mir::{
    AccessElisionReason, Body, ElidedAccess, FuncRef, Operand, Place, Projection, Rvalue, Stmt, StmtKind,
    Terminator,
};
use ember_types::{ClassId, TyKind, TypeTable};

/// Remove only accesses for which `[EXC-3]`'s `unique_handle` condition is
/// directly established.  The removed sites are retained on the MIR body so
/// `[EXC-3a]` can report them in the `[EFF-10]` side table.
pub fn elide_static_accesses_all(bodies: &mut [Body], types: &TypeTable) -> usize {
    bodies
        .iter_mut()
        .map(|body| elide_static_accesses(body, types))
        .sum()
}

fn elide_static_accesses(body: &mut Body, types: &TypeTable) -> usize {
    let mut remove = Vec::new();
    let mut records = Vec::new();

    for (block_index, block) in body.blocks.iter().enumerate() {
        for (begin_index, statement) in block.stmts.iter().enumerate() {
            let StmtKind::BeginAccess { place, mutable } = &statement.kind else {
                continue;
            };
            if !*mutable {
                continue;
            }
            let Some((local, allowed_reborrow)) = unique_owner_local(body, types, place) else {
                continue;
            };
            let Some((end_block, end_index)) = matching_end(body, block_index, begin_index, place, *mutable) else {
                continue;
            };
            if !unique_handle(body, block_index, local, allowed_reborrow) {
                continue;
            }
            remove.push((block_index, begin_index, end_block, end_index));
            records.push(ElidedAccess {
                span: statement.span,
                reason: AccessElisionReason::UniqueHandle,
            });
        }
    }

    remove.sort_by(|left, right| {
        (right.2, right.3, right.0, right.1).cmp(&(left.2, left.3, left.0, left.1))
    });
    for (begin_block, begin_index, end_block, end_index) in remove {
        if begin_block == end_block {
            let statements = &mut body.blocks[begin_block].stmts;
            statements.remove(end_index);
            statements.remove(begin_index);
        } else {
            body.blocks[end_block].stmts.remove(end_index);
            body.blocks[begin_block].stmts.remove(begin_index);
        }
    }
    let count = records.len();
    body.elided_accesses.extend(records);
    count
}

fn matching_end(
    body: &Body,
    block_index: usize,
    begin_index: usize,
    place: &Place,
    mutable: bool,
) -> Option<(usize, usize)> {
    let block = &body.blocks[block_index];
    let mut nested = 0usize;
    for (index, statement) in block.stmts.iter().enumerate().skip(begin_index + 1) {
        match &statement.kind {
            StmtKind::BeginAccess { .. } | StmtKind::BeginAccessTransfer { .. } => nested += 1,
            StmtKind::EndAccess { place: end_place, mutable: end_mutable }
            | StmtKind::EndAccessTransfer { place: end_place, mutable: end_mutable } => {
                if nested != 0 {
                    nested -= 1;
                } else if end_place == place && *end_mutable == mutable {
                    return Some((block_index, index));
                }
            }
            _ => {}
        }
    }
    let successor = match &block.terminator {
        // Preserve the existing direct-call case exactly. A call may close an
        // access only when it immediately follows the begin.
        Terminator::Call { next, .. } if begin_index + 1 == block.stmts.len() => *next,
        // An assertion has a single success successor and does not invoke
        // user code. Its failure path aborts, so a matching end at the start
        // of the success block still forms a closed interval.
        Terminator::Assert { next, .. } => *next,
        _ => return None,
    };
    let successor_block = body.blocks.get(successor.0 as usize)?;
    // The end may follow plain statements (a `print` argument copied out,
    // D-286): an assignment or a storage marker runs no user code. A `drop`
    // may, and anything else ends the search.
    for (index, statement) in successor_block.stmts.iter().enumerate() {
        match &statement.kind {
            StmtKind::EndAccess { place: end_place, mutable: end_mutable } => {
                return (end_place == place && *end_mutable == mutable).then_some((successor.0 as usize, index));
            }
            StmtKind::Assign { .. } | StmtKind::StorageLive(_) | StmtKind::StorageDead(_) | StmtKind::Nop => {}
            _ => return None,
        }
    }
    None
}

/// The source owner a closed access can use for `[EXC-3]` proof. Classes
/// already begin directly on their local handle. `Shared.get_mut()` must first
/// form a normal `ref mut Shared[T]`; accepting only the exact compiler-created
/// temporary preserves that same proof without treating arbitrary references as
/// unique handles.
fn unique_owner_local(
    body: &Body,
    types: &TypeTable,
    place: &Place,
) -> Option<(ember_mir::LocalId, Option<ember_mir::LocalId>)> {
    if place.projection.is_empty()
        && matches!(types.kind(body.local(place.local).ty), TyKind::Class(_))
    {
        return Some((place.local, None));
    }
    if !matches!(place.projection.as_slice(), [ember_mir::Projection::Deref]) {
        return None;
    }
    let temporary = place.local;
    let mut source = None;
    for block in &body.blocks {
        for statement in &block.stmts {
            let StmtKind::Assign {
                place: destination,
                rvalue,
            } = &statement.kind
            else {
                continue;
            };
            if destination != &Place::local(temporary) {
                continue;
            }
            let Rvalue::Ref {
                place: borrowed,
                mutable: true,
            } = rvalue
            else {
                return None;
            };
            if !borrowed.projection.is_empty() {
                return None;
            }
            if !is_shared_owner(body, types, borrowed.local) {
                return None;
            }
            if source.replace(borrowed.local).is_some() {
                return None;
            }
        }
    }
    source.map(|local| (local, Some(temporary)))
}

fn is_shared_owner(body: &Body, types: &TypeTable, local: ember_mir::LocalId) -> bool {
    let TyKind::Struct(id) = types.kind(body.local(local).ty) else {
        return false;
    };
    types.struct_def(*id)
        .origin
        .as_ref()
        .is_some_and(|(name, arguments)| name.is("Shared") && arguments.len() == 1)
}

fn unique_handle(
    body: &Body,
    access_block: usize,
    root: ember_mir::LocalId,
    allowed_reborrow: Option<ember_mir::LocalId>,
) -> bool {
    for (block_index, block) in body.blocks.iter().enumerate() {
        for statement in &block.stmts {
            if matches!(statement.kind, StmtKind::BeginAccess { .. } | StmtKind::EndAccess { .. } | StmtKind::Drop { .. }) {
                continue;
            }
            match &statement.kind {
                StmtKind::Assign { place, rvalue } => {
                    if place == &Place::local(root) {
                        return false;
                    }
                    if matches!(
                        rvalue,
                        Rvalue::Ref {
                            place: borrowed,
                            mutable: true,
                        }
                            if borrowed == &Place::local(root)
                                && Some(place.local) == allowed_reborrow
                                && place.projection.is_empty()
                    ) {
                        continue;
                    }
                    if rvalue_uses_root(rvalue, root) {
                        return false;
                    }
                }
                StmtKind::CheckedBinaryOp { dest, lhs, rhs, .. } => {
                    if dest == &Place::local(root)
                        || operand_uses_root(lhs, root)
                        || operand_uses_root(rhs, root)
                    {
                        return false;
                    }
                }
                StmtKind::StorageLive(_)
                | StmtKind::StorageDead(_)
                | StmtKind::Nop
                | StmtKind::BeginAccess { .. }
                | StmtKind::BeginAccessTransfer { .. }
                | StmtKind::EndAccess { .. }
                | StmtKind::EndAccessTransfer { .. }
                | StmtKind::Drop { .. } => {}
            }
        }
        if terminator_uses_root(&block.terminator, root, block_index == access_block) {
            return false;
        }
    }
    true
}

fn rvalue_uses_root(rvalue: &Rvalue, root: ember_mir::LocalId) -> bool {
    match rvalue {
        Rvalue::Use(operand) | Rvalue::UnaryOp { operand, .. } | Rvalue::Cast { operand, .. } => {
            operand_uses_root(operand, root)
        }
        Rvalue::BinaryOp { lhs, rhs, .. } => {
            operand_uses_root(lhs, root) || operand_uses_root(rhs, root)
        }
        Rvalue::Aggregate { operands, .. } => operands.iter().any(|o| operand_uses_root(o, root)),
        Rvalue::Repeat { value, .. } => operand_uses_root(value, root),
        Rvalue::Discriminant(place) | Rvalue::Ref { place, .. } => place == &Place::local(root),
    }
}

fn operand_uses_root(operand: &Operand, root: ember_mir::LocalId) -> bool {
    matches!(operand, Operand::Copy(place) | Operand::Move(place) if place == &Place::local(root))
}

fn terminator_uses_root(
    terminator: &Terminator,
    root: ember_mir::LocalId,
    destination_is_inside_access: bool,
) -> bool {
    match terminator {
        Terminator::SwitchInt { discr, .. } => operand_uses_root(discr, root),
        Terminator::Call { func, args, dest, .. } => {
            let destination_is_root = dest == &Place::local(root);
            let callee_uses_root = match func {
                ember_mir::FuncRef::Indirect { operand, .. } => operand_uses_root(operand, root),
                _ => false,
            };
            // A constructor may initialize the root before the access, so a
            // call destination alone is not an escape. Passing the root as a
            // callee/argument is always opaque and therefore disqualifying.
            (destination_is_root && destination_is_inside_access)
                || (!destination_is_root && callee_uses_root)
                || args.iter().any(|arg| operand_uses_root(arg, root))
        }
        Terminator::Assert { cond, msg, .. } => {
            operand_uses_root(cond, root)
                || match msg {
                    ember_mir::AssertKind::Bounds { len, index } => {
                        operand_uses_root(len, root) || operand_uses_root(index, root)
                    }
                    ember_mir::AssertKind::RefCellBorrow { file, line } => {
                        operand_uses_root(file, root) || operand_uses_root(line, root)
                    }
                    ember_mir::AssertKind::Overflow(_)
                    | ember_mir::AssertKind::DivisionByZero
                    | ember_mir::AssertKind::SignedDivisionOverflow
                    | ember_mir::AssertKind::ShiftTooLarge
                    | ember_mir::AssertKind::Downcast => false,
                    ember_mir::AssertKind::Panic { message } => operand_uses_root(message, root),
                }
        }
        Terminator::Goto(_) | Terminator::Return | Terminator::Unreachable => false,
    }
}

/// `[EXC-3]`, `[EXC-3a]` — remove every check (an access begun and ended back
/// to back, `[EXC-19]`) that nothing held anywhere in the program could make
/// fail, and record each. An access is held when other code runs between its
/// begin and its end. A held access is recorded against the field it covers
/// (the class declaring it, and its name) or, for a whole object, against the
/// object's class family: its dynamic class is its static class or one
/// derived from it, and a family shares the root of its base chain. A read
/// check needs no held write that could cover it; a write check, no held
/// access. A `dyn` or interface anywhere in the program turns the pass off:
/// those reach accesses taken on concrete classes the pass cannot see. A
/// class with a `drop` holds a write of its whole object while it runs (the
/// runtime begins it, D-453), so its family counts as holding one.
pub fn remove_never_firing_checks_all(bodies: &mut [Body], types: &TypeTable) -> usize {
    let mut held_write: HashSet<(ClassId, String)> = HashSet::new();
    let mut held_any: HashSet<(ClassId, String)> = HashSet::new();
    let mut held_write_family: HashSet<ClassId> = HashSet::new();
    let mut held_any_family: HashSet<ClassId> = HashSet::new();
    let mut object_write_family: HashSet<ClassId> = types
        .classes()
        .filter(|(_, def)| def.has_drop && !def.is_sync)
        .map(|(id, _)| class_family(types, id))
        .collect();
    let mut object_any_family: HashSet<ClassId> = object_write_family.clone();
    for body in bodies.iter() {
        for basic_block in &body.blocks {
            if let Terminator::Call { func: FuncRef::Interface { .. } | FuncRef::DynBoxNew { .. }, .. } = basic_block.terminator {
                return 0;
            }
            for (index, stmt) in basic_block.stmts.iter().enumerate() {
                let StmtKind::BeginAccess { place, mutable } = &stmt.kind else { continue };
                let target = access_target(body, types, place);
                if matches!(target, AccessTarget::Unknown) {
                    return 0;
                }
                if is_check(&basic_block.stmts, index) {
                    continue;
                }
                match target {
                    AccessTarget::Field { key, family } => {
                        if *mutable {
                            held_write.insert(key.clone());
                            held_write_family.insert(family);
                        }
                        held_any.insert(key);
                        held_any_family.insert(family);
                    }
                    AccessTarget::Object { family } => {
                        if *mutable {
                            object_write_family.insert(family);
                        }
                        object_any_family.insert(family);
                    }
                    AccessTarget::Unknown | AccessTarget::Other => {}
                }
            }
        }
    }
    let mut removed = 0;
    for body in bodies.iter_mut() {
        for block in 0..body.blocks.len() {
            let mut index = 0;
            while index < body.blocks[block].stmts.len() {
                if !is_check(&body.blocks[block].stmts, index) {
                    index += 1;
                    continue;
                }
                let stmt = &body.blocks[block].stmts[index];
                let StmtKind::BeginAccess { place, mutable } = &stmt.kind else { unreachable!("a check begins") };
                let (mutable, span) = (*mutable, stmt.span);
                let never_fires = match access_target(body, types, place) {
                    AccessTarget::Field { key, family } if mutable => {
                        !held_any.contains(&key) && !object_any_family.contains(&family)
                    }
                    AccessTarget::Field { key, family } => {
                        !held_write.contains(&key) && !object_write_family.contains(&family)
                    }
                    AccessTarget::Object { family } if mutable => {
                        !held_any_family.contains(&family) && !object_any_family.contains(&family)
                    }
                    AccessTarget::Object { family } => {
                        !held_write_family.contains(&family) && !object_write_family.contains(&family)
                    }
                    AccessTarget::Unknown | AccessTarget::Other => false,
                };
                if never_fires {
                    body.blocks[block].stmts.drain(index..index + 2);
                    body.elided_accesses.push(ElidedAccess { span, reason: AccessElisionReason::NoConflictingHold });
                    removed += 1;
                } else {
                    index += 2;
                }
            }
        }
    }
    removed
}

/// An access begun and ended back to back: `[EXC-19]`'s check.
fn is_check(stmts: &[Stmt], index: usize) -> bool {
    let StmtKind::BeginAccess { place, mutable } = &stmts[index].kind else { return false };
    matches!(stmts.get(index + 1).map(|next| &next.kind),
        Some(StmtKind::EndAccess { place: end, mutable: end_mutable }) if end == place && end_mutable == mutable)
}

/// What an access is to: a class field, keyed by the class declaring it and
/// its name; a whole class object, by its family; something whose class the
/// pass cannot know (a `dyn` or interface); or neither (a `Shared` payload).
enum AccessTarget {
    Field { key: (ClassId, String), family: ClassId },
    Object { family: ClassId },
    Unknown,
    Other,
}

fn access_target(body: &Body, types: &TypeTable, place: &Place) -> AccessTarget {
    match types.kind(crate::regions::place_type(body, types, place)) {
        TyKind::Class(id) => return AccessTarget::Object { family: class_family(types, *id) },
        TyKind::ClassInterface(_) | TyKind::Dyn { .. } => return AccessTarget::Unknown,
        TyKind::Ref { inner, .. } if matches!(types.kind(*inner), TyKind::Dyn { .. }) => return AccessTarget::Unknown,
        TyKind::Struct(id) if matches!(&types.struct_def(*id).origin, Some((name, _)) if name.is("Box")) => {
            return AccessTarget::Unknown;
        }
        _ => {}
    }
    let Some((Projection::Field(index), prefix)) = place.projection.split_last() else { return AccessTarget::Other };
    let handle = Place { local: place.local, projection: prefix.to_vec() };
    let TyKind::Class(id) = *types.kind(crate::regions::place_type(body, types, &handle)) else {
        return AccessTarget::Other;
    };
    // A field without its own word (a `Shared[T]`) is its payload's access.
    if !types.class_field_has_access_word(id, *index) {
        return AccessTarget::Other;
    }
    match types.class_field_at_info(id, *index) {
        Some((owner, field)) => {
            AccessTarget::Field { key: (owner, field.name.to_string()), family: class_family(types, owner) }
        }
        None => AccessTarget::Unknown,
    }
}

/// The root of `id`'s base chain: every class an object of static class `id`
/// can dynamically be shares it.
pub(crate) fn class_family(types: &TypeTable, id: ClassId) -> ClassId {
    let mut family = id;
    while let Some(base) = types.class_def(family).base {
        family = base;
    }
    family
}
