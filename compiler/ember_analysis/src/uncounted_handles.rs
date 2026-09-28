//! `[RC-3]` — a class handle copied out of a list the function owns needs no
//! count of its own while the list keeps holding the object.
//!
//! `t = things[i]` retains the object and `t`'s drop releases it. When nothing
//! can take the object out of `things` while `t` lives, the list's own count
//! keeps the object alive, so the pair cancels: removing it moves no `drop`,
//! no `Weak.upgrade` outcome and no foreign release.
//!
//! Nothing can take it out when, on every path from the copy to `t`'s drop,
//! this function neither writes, moves nor drops `things` (a field of an
//! element's object may be written) nor takes a mutable reference to it. No
//! other code can reach `things` then: the borrow checker allows no mutable
//! reference to it that lives across the copy, and a raw pointer may not be
//! used after the reference it came from (`[UNS-4]`).
//!
//! `t` must stay the same handle for the whole time: it is never moved, never
//! assigned anything but such a copy, and lent mutably only where the callee
//! cannot re-point it — a `mut self` receiver (`E2103`) or a parameter that is
//! not `mut`. Copies of `t` count themselves as usual.

use std::collections::{HashMap, HashSet};

use ember_mir::{Body, FuncRef, LocalId, LocalKind, Operand, ParameterMode, Place, Projection, Rvalue, StmtKind, Terminator};
use ember_types::{TyKind, TypeTable};

use crate::long_access_lint::successors;
use crate::regions::place_type;

/// What a direct callee lets its caller lend it: its parameter modes, and
/// whether its first parameter is a `mut self` receiver.
struct Callee {
    modes: Vec<ParameterMode>,
    mut_self: bool,
}

/// Mark every such handle in `bodies` (`Body::uncounted_handles`); returns how
/// many were marked.
pub fn mark_uncounted_handles_all(bodies: &mut [Body], types: &TypeTable) -> usize {
    let foreign: HashSet<&str> =
        bodies.iter().filter(|body| body.is_extern_declaration).map(|body| body.symbol.as_str()).collect();
    let callees: HashMap<String, Callee> = bodies
        .iter()
        .filter(|body| !foreign.contains(body.symbol.as_str()))
        .map(|body| (body.symbol.clone(), Callee { modes: body.param_modes.clone(), mut_self: body.mut_self }))
        .collect();
    let mut marked = 0;
    for body in bodies.iter_mut() {
        let handles: Vec<LocalId> = (body.arg_count as u32 + 1..body.locals.len() as u32)
            .map(LocalId)
            .filter(|&local| uncounted(body, types, &callees, local))
            .collect();
        marked += handles.len();
        body.uncounted_handles = handles;
    }
    marked
}

fn whole(place: &Place, local: LocalId) -> bool {
    place.local == local && place.projection.is_empty()
}

/// Whether `t` is a class handle whose every value is an uncounted copy of a
/// list element.
fn uncounted(body: &Body, types: &TypeTable, callees: &HashMap<String, Callee>, t: LocalId) -> bool {
    if !matches!(types.kind(body.local(t).ty), TyKind::Class(_)) {
        return false;
    }
    // Every assignment of `t` copies an element; every drop is unconditional.
    let mut copies = Vec::new();
    let mut drops = HashSet::new();
    for (b, block) in body.blocks.iter().enumerate() {
        for (s, stmt) in block.stmts.iter().enumerate() {
            match &stmt.kind {
                StmtKind::Assign { place, rvalue } if whole(place, t) => match element_of_owned_list(body, types, rvalue) {
                    Some(list) => copies.push((b, s, list)),
                    None => return false,
                },
                StmtKind::CheckedBinaryOp { dest, .. } if whole(dest, t) => return false,
                StmtKind::Drop { place, flag, .. } if whole(place, t) => {
                    if flag.is_some() {
                        return false;
                    }
                    drops.insert((b, s));
                }
                _ => {}
            }
        }
        if let Terminator::Call { dest, .. } = &block.terminator {
            if whole(dest, t) {
                return false;
            }
        }
    }
    if copies.is_empty() || !stays_the_same_handle(body, types, callees, t) {
        return false;
    }
    // From each copy to `t`'s drop, the list keeps the object; and every drop
    // of `t` ends one of these copies.
    let mut reached = HashSet::new();
    for (b, s, lists) in &copies {
        if !list_kept(body, (*b, *s + 1), t, lists, &mut reached) {
            return false;
        }
    }
    reached == drops
}

/// The locals that must stay unchanged for the element `rvalue` copies to
/// stay in its list: `copy list[i]`, where `list` is a user-written local or
/// `owned` parameter holding an `Array` or fixed array of class handles, and
/// never a bitwise copy of another place. The one exception is the header a
/// loop's unchecked copy reads its elements through (`[OPT-2]`), a copy of
/// such a list: then both.
fn element_of_owned_list(body: &Body, types: &TypeTable, rvalue: &Rvalue) -> Option<Vec<LocalId>> {
    let Rvalue::Use(Operand::Copy(place)) = rvalue else { return None };
    let [Projection::Index(_) | Projection::ConstIndex(_)] = place.projection.as_slice() else { return None };
    let list = place.local;
    let elem = match types.kind(body.local(list).ty) {
        TyKind::Vec { elem, .. } | TyKind::Array { elem, .. } => *elem,
        _ => return None,
    };
    if !matches!(types.kind(elem), TyKind::Class(_)) {
        return None;
    }
    // The places copied whole into `list`.
    let copies: Vec<&Place> = body
        .blocks
        .iter()
        .flat_map(|block| &block.stmts)
        .filter_map(|stmt| match &stmt.kind {
            StmtKind::Assign { place, rvalue: Rvalue::Use(Operand::Copy(from)) } if whole(place, list) => Some(from),
            _ => None,
        })
        .collect();
    let owned = |local: LocalId| match body.local(local).kind {
        LocalKind::User => true,
        LocalKind::Arg => body.param_modes.get(local.0 as usize - 1) == Some(&ParameterMode::Owned),
        LocalKind::Return | LocalKind::Temp => false,
    };
    if owned(list) && copies.is_empty() {
        return Some(vec![list]);
    }
    // A header copy: a temporary set only by copying one whole owned list.
    let origin = copies.first()?.local;
    let one_origin = copies.iter().all(|from| from.projection.is_empty() && from.local == origin);
    let set_otherwise = body.blocks.iter().any(|block| {
        block.stmts.iter().any(|stmt| match &stmt.kind {
            StmtKind::Assign { place, rvalue } if whole(place, list) => !matches!(rvalue, Rvalue::Use(Operand::Copy(_))),
            StmtKind::CheckedBinaryOp { dest, .. } => whole(dest, list),
            _ => false,
        }) || matches!(&block.terminator, Terminator::Call { dest, .. } if whole(dest, list))
    });
    let origin_copied = body.blocks.iter().flat_map(|block| &block.stmts).any(|stmt| {
        matches!(&stmt.kind, StmtKind::Assign { place, rvalue: Rvalue::Use(Operand::Copy(_)) } if whole(place, origin))
    });
    (body.local(list).kind == LocalKind::Temp && one_origin && !set_otherwise && owned(origin) && !origin_copied)
        .then(|| vec![list, origin])
}

/// Whether every use of `t` leaves it holding the same object: no move of it,
/// and every reference to it (a local assigned only `ref t` or `ref mut t`)
/// is only passed to calls that cannot re-point it through that reference.
fn stays_the_same_handle(body: &Body, types: &TypeTable, callees: &HashMap<String, Callee>, t: LocalId) -> bool {
    // The locals holding a reference to `t`, and whether it is mutable.
    let mut refs: HashMap<LocalId, bool> = HashMap::new();
    let mut other_values = HashSet::new();
    for block in &body.blocks {
        for stmt in &block.stmts {
            let StmtKind::Assign { place, rvalue } = &stmt.kind else { continue };
            match rvalue {
                Rvalue::Ref { place: target, mutable } if whole(target, t) => {
                    if !place.projection.is_empty() || body.local(place.local).kind == LocalKind::User {
                        return false;
                    }
                    *refs.entry(place.local).or_default() |= *mutable;
                }
                _ if place.projection.is_empty() => {
                    other_values.insert(place.local);
                }
                _ => {}
            }
        }
        if let Terminator::Call { dest, .. } = &block.terminator {
            other_values.insert(dest.local);
        }
    }
    if refs.keys().any(|r| other_values.contains(r)) {
        return false;
    }
    // `t` itself, or the handle a reference to it points at.
    let is_handle = |place: &Place| {
        whole(place, t) || (refs.contains_key(&place.local) && place.projection == [Projection::Deref])
    };
    let moves = |operand: &Operand| matches!(operand, Operand::Move(place) if is_handle(place) || refs.contains_key(&place.local));
    for block in &body.blocks {
        for stmt in &block.stmts {
            match &stmt.kind {
                StmtKind::Assign { place, rvalue } => {
                    if !whole(place, t) && is_handle(place) {
                        return false;
                    }
                    let bad = match rvalue {
                        Rvalue::Use(value) | Rvalue::UnaryOp { operand: value, .. } | Rvalue::Cast { operand: value, .. } => {
                            moves(value) || copies_ref(value, &refs)
                        }
                        Rvalue::BinaryOp { lhs, rhs, .. } => [lhs, rhs].iter().any(|v| moves(v) || copies_ref(v, &refs)),
                        Rvalue::Aggregate { operands, .. } => operands.iter().any(|v| moves(v) || copies_ref(v, &refs)),
                        Rvalue::Repeat { value, .. } => moves(value) || copies_ref(value, &refs),
                        Rvalue::Ref { place: target, mutable } => {
                            (*mutable && is_handle(target) && !whole(target, t)) || refs.contains_key(&target.local) && target.projection.is_empty()
                        }
                        Rvalue::Discriminant(_) => false,
                    };
                    if bad {
                        return false;
                    }
                }
                StmtKind::CheckedBinaryOp { dest, lhs, rhs, .. } => {
                    if is_handle(dest) || [lhs, rhs].iter().any(|v| moves(v) || copies_ref(v, &refs)) {
                        return false;
                    }
                }
                StmtKind::Drop { place, .. } => {
                    if !whole(place, t) && is_handle(place) {
                        return false;
                    }
                }
                _ => {}
            }
        }
        match &block.terminator {
            Terminator::Call { func, args, dest, .. } => {
                if is_handle(dest) || args.iter().any(|arg| moves(arg)) {
                    return false;
                }
                for (k, arg) in args.iter().enumerate() {
                    let Operand::Copy(place) = arg else { continue };
                    let Some(&mutable) = refs.get(&place.local).filter(|_| place.projection.is_empty()) else { continue };
                    // A reference the call returns could carry `t` out of it.
                    if matches!(types.kind(place_type(body, types, dest)), TyKind::Ref { .. } | TyKind::Ptr { .. }) {
                        return false;
                    }
                    let lends_safely = match func {
                        FuncRef::Direct { symbol, .. } => callees.get(symbol).is_some_and(|callee| {
                            (k == 0 && callee.mut_self) || !mutable && callee.modes.get(k) != Some(&ParameterMode::Mut)
                        }),
                        FuncRef::Virtual { param_modes, .. } => {
                            k == 0 || !mutable && param_modes.get(k) != Some(&ParameterMode::Mut)
                        }
                        _ => !mutable,
                    };
                    if !lends_safely {
                        return false;
                    }
                }
            }
            Terminator::SwitchInt { discr, .. } if moves(discr) || copies_ref(discr, &refs) => return false,
            _ => {}
        }
    }
    true
}

/// Whether `operand` copies a reference to `t` somewhere other than a call.
fn copies_ref(operand: &Operand, refs: &HashMap<LocalId, bool>) -> bool {
    matches!(operand, Operand::Copy(place) if place.projection.is_empty() && refs.contains_key(&place.local))
}

/// Walk every path from `start` to a drop of `t`, recording each drop
/// reached; `false` when a path writes, moves, drops or mutably lends `list`,
/// assigns `t` again, or returns first.
fn list_kept(
    body: &Body,
    start: (usize, usize),
    t: LocalId,
    lists: &[LocalId],
    reached: &mut HashSet<(usize, usize)>,
) -> bool {
    // A place of a list itself; a place inside an element's object is not.
    let of_list = |place: &Place| {
        lists.contains(&place.local)
            && !(place.projection.len() >= 2 && matches!(place.projection[0], Projection::Index(_) | Projection::ConstIndex(_)))
    };
    let moves = |operand: &Operand| matches!(operand, Operand::Move(place) if lists.contains(&place.local));
    let mut stack = vec![start];
    let mut seen = HashSet::new();
    while let Some((b, from)) = stack.pop() {
        let block = &body.blocks[b];
        let mut ended = false;
        for (s, stmt) in block.stmts.iter().enumerate().skip(from) {
            let touches = match &stmt.kind {
                StmtKind::Drop { place, .. } if whole(place, t) => {
                    reached.insert((b, s));
                    ended = true;
                    break;
                }
                StmtKind::Assign { place, .. } if whole(place, t) => true,
                StmtKind::Assign { place, rvalue } => {
                    of_list(place)
                        || match rvalue {
                            Rvalue::Use(value) | Rvalue::UnaryOp { operand: value, .. } | Rvalue::Cast { operand: value, .. } => moves(value),
                            Rvalue::Repeat { value, .. } => moves(value),
                            Rvalue::BinaryOp { lhs, rhs, .. } => moves(lhs) || moves(rhs),
                            Rvalue::Aggregate { operands, .. } => operands.iter().any(moves),
                            Rvalue::Ref { place, mutable } => *mutable && of_list(place),
                            Rvalue::Discriminant(_) => false,
                        }
                }
                StmtKind::CheckedBinaryOp { dest, overflow, .. } => of_list(dest) || of_list(overflow),
                StmtKind::Drop { place, .. } => of_list(place),
                StmtKind::StorageLive(local) | StmtKind::StorageDead(local) => lists.contains(local),
                _ => false,
            };
            if touches {
                return false;
            }
        }
        if ended {
            continue;
        }
        match &block.terminator {
            Terminator::Call { args, dest, .. } if of_list(dest) || args.iter().any(moves) => return false,
            Terminator::Return => return false,
            _ => {}
        }
        for next in successors(&block.terminator) {
            if seen.insert(next) {
                stack.push((next, 0));
            }
        }
    }
    true
}
