//! At `main`'s return the process ends. A drop there that can only free
//! memory changes nothing a program can observe (`[PHIL-5]`): the operating
//! system takes the memory back. Freeing a million objects one by one costs
//! the time a C program that leaves them to the operating system does not
//! spend (calls through an interface: 1.24x C, 21 of its 27 ms). A drop that
//! runs a `drop` method anywhere in what it reaches stays, as does every
//! drop in debug builds, whose leak check counts what is left.

use std::collections::HashSet;

use ember_mir::{Body, StmtKind, Terminator};
use ember_types::{ClassId, Ty, TyKind, TypeTable};

use crate::regions::place_type;

/// Removes the drops that end `main` (the body named `main`) when each can
/// only free memory. Returns how many went.
pub fn skip_exit_drops_all(bodies: &mut [Body], types: &TypeTable, main: &str) -> usize {
    bodies.iter_mut().filter(|body| body.symbol == main).map(|body| skip_exit_drops(body, types)).sum()
}

fn skip_exit_drops(body: &mut Body, types: &TypeTable) -> usize {
    let n = body.blocks.len();
    let mut predecessors = vec![Vec::new(); n];
    for (block, data) in body.blocks.iter().enumerate() {
        if let Terminator::Goto(next) = &data.terminator {
            predecessors[next.0 as usize].push(block);
        }
    }
    let mut skipped = 0;
    // From each return back: a block's last statements, while each is a
    // drop that frees memory only or bookkeeping; a block all of whose
    // statements went, and that only a jump reaches, lets its predecessor's
    // tail go too.
    let mut work: Vec<usize> = (0..n).filter(|&block| matches!(body.blocks[block].terminator, Terminator::Return)).collect();
    let mut seen: HashSet<usize> = work.iter().copied().collect();
    while let Some(block) = work.pop() {
        let mut whole = true;
        for index in (0..body.blocks[block].stmts.len()).rev() {
            let stmt = &body.blocks[block].stmts[index];
            match &stmt.kind {
                StmtKind::StorageLive(_) | StmtKind::StorageDead(_) | StmtKind::Nop => {}
                StmtKind::Drop { place, .. } if memory_only(types, place_type(body, types, place), &mut HashSet::new()) => {
                    body.blocks[block].stmts[index].kind = StmtKind::Nop;
                    skipped += 1;
                }
                _ => {
                    whole = false;
                    break;
                }
            }
        }
        if whole {
            for &from in &predecessors[block] {
                if seen.insert(from) {
                    work.push(from);
                }
            }
        }
    }
    skipped
}

/// Whether dropping a `ty` can only free memory: no `drop` method runs in
/// anything it reaches, and no other thread can hold what it releases.
fn memory_only(types: &TypeTable, ty: Ty, seen: &mut HashSet<Ty>) -> bool {
    if !seen.insert(ty) {
        return true;
    }
    match types.kind(ty) {
        TyKind::Struct(id) => {
            let def = types.struct_def(*id);
            if def.has_drop {
                return false;
            }
            match &def.origin {
                // A weak handle never drops its target.
                Some((name, args)) if name.is("Weak") && args.len() == 1 => true,
                // The last shared handle, or a box, drops what it holds.
                Some((name, args)) if (name.is("Shared") || name.is("Box")) && args.len() == 1 => {
                    memory_only(types, args[0], seen)
                }
                Some((name, _)) if name.is("SyncShared") => false,
                _ => !def.drops_fields || def.fields.iter().all(|field| memory_only(types, field.ty, seen)),
            }
        }
        TyKind::Enum(id) => {
            let def = types.enum_def(*id);
            !def.has_drop && def.variants.iter().all(|variant| variant.fields.iter().all(|field| memory_only(types, field.ty, seen)))
        }
        TyKind::Tuple(items) => items.iter().all(|&item| memory_only(types, item, seen)),
        TyKind::Array { elem, .. } | TyKind::Vec { elem, .. } => memory_only(types, *elem, seen),
        // The handle may point at the class or any class derived from it.
        TyKind::Class(id) => {
            types.runtime_classes().all(|(class, _)| !derives(types, class, *id) || class_memory_only(types, class, seen))
        }
        // Any class of the program may implement the interface.
        TyKind::ClassInterface(_) => types.runtime_classes().all(|(class, _)| class_memory_only(types, class, seen)),
        TyKind::Dyn { .. } => false,
        _ => !types.needs_drop(ty),
    }
}

fn derives(types: &TypeTable, class: ClassId, from: ClassId) -> bool {
    let mut current = Some(class);
    while let Some(id) = current {
        if id == from {
            return true;
        }
        current = types.class_def(id).base;
    }
    false
}

/// A class, with the classes it derives from, has no `drop` method, cannot
/// be shared across threads, and its fields free memory only.
fn class_memory_only(types: &TypeTable, class: ClassId, seen: &mut HashSet<Ty>) -> bool {
    let mut current = Some(class);
    while let Some(id) = current {
        let def = types.class_def(id);
        if def.has_drop || def.is_sync || !def.fields.iter().all(|field| memory_only(types, field.ty, seen)) {
            return false;
        }
        current = def.base;
    }
    true
}
