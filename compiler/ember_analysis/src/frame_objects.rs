//! `[OPT-1]` — stack promotion. A class object no handle of which outlives the function that
//! makes it is placed in that function's frame, with the same header, instead of on the heap: its
//! counts, its exclusivity words and its `drop` at the last release are what they would be, and only
//! the memory is the frame's (the runtime's `obj_new_frame` marks the object, and its last release
//! frees nothing). This cannot be observed.
//!
//! A local `L` that a class constructor sets (`L = class()`) is promoted when escape analysis proves:
//! * every local that ever holds the handle (`L`, and any `M = L` or upcast copy of it, the
//!   *aliases*) is used only to reach the object's fields, to compare, to be dropped, or as an
//!   argument of a direct call whose parameter does not escape its callee (the same analysis,
//!   to a fixpoint over the call graph). A handle stored anywhere but a local, returned, captured,
//!   borrowed as a place, turned into an interface handle or a `Weak`, or given to a built-in, a
//!   callable value, a virtual or interface method or a foreign function, escapes;
//! * where the constructor runs again (a loop), no alias holding the object made before is still
//!   to be used (a counted alias is dropped, a use, before its object is gone): the storage is
//!   free again. When the frame goes every counted alias has been dropped, so every object made
//!   in it has had its last release;
//! * the function cannot come back to itself (a cycle of direct calls, or a call the compiler
//!   cannot follow), and the object is at most `MOST_BYTES`, at most `MOST_PER_BODY` a function:
//!   a frame grows only by a bound the heap would have taken, so a recursion's stack stays as it
//!   was (`[RT-12]`).

use std::collections::{HashMap, HashSet};

use ember_mir::{
    BasicBlockId, Body, Builtin, CastKind, FuncRef, LocalId, Operand, Place, Projection, Rvalue, StmtKind, Terminator,
    RETURN_LOCAL,
};
use ember_types::{TyKind, TypeTable};

const MOST_BYTES: u64 = 256;
const MOST_PER_BODY: usize = 4;

/// Place every such object of `bodies` in its function's frame (`Body::frame_objects`); returns
/// how many were placed.
pub fn place_frame_objects_all(bodies: &mut [Body], types: &TypeTable) -> usize {
    let escapes = parameter_escapes(bodies);
    let open = crate::stack_lists::may_come_back(bodies);
    // A class's `drop` runs at its object's last release, from the runtime, not from a call this
    // analysis follows: the object is promoted only if no `drop` of its class or of a base lets
    // `self` escape (a `Weak` of it, say, would outlive the frame).
    let mut drop_keeps: HashMap<ember_types::ClassId, bool> = HashMap::new();
    for body in bodies.iter() {
        if let Some(class) = body.class_owner
            && body.name == "drop"
        {
            let keeps = escapes.get(&body.symbol).and_then(|params| params.first()).is_none_or(|escapes| *escapes);
            *drop_keeps.entry(class).or_default() |= keeps;
        }
    }
    let drop_escapes = |mut class: ember_types::ClassId| loop {
        if drop_keeps.get(&class).copied().unwrap_or(false) {
            return true;
        }
        match types.class_def(class).base {
            Some(base) => class = base,
            None => return false,
        }
    };
    let mut placed = 0;
    for (index, body) in bodies.iter_mut().enumerate() {
        if open[index] || body.is_extern_declaration || body.is_abstract || body.closure_environment.is_some() {
            continue;
        }
        let mut chosen: Vec<LocalId> = Vec::new();
        for local in constructed_locals(body) {
            if chosen.len() == MOST_PER_BODY {
                break;
            }
            let TyKind::Class(class) = types.kind(body.local(local).ty) else { continue };
            if class_bytes(types, *class) > MOST_BYTES || drop_escapes(*class) {
                continue;
            }
            let aliases = aliases_of(body, local);
            if !escapes_body(body, &aliases, &escapes) && released_where_needed(body, &aliases, local) {
                chosen.push(local);
            }
        }
        placed += chosen.len();
        body.frame_objects = chosen;
    }
    placed
}

/// The size of a class's object as its fields add up, header aside: a bound, not a layout.
fn class_bytes(types: &TypeTable, class: ember_types::ClassId) -> u64 {
    let mut bytes = 0u64;
    let mut index = 0;
    while let Some(field) = types.class_field_at(class, index) {
        bytes += types.layout(field.ty).size.max(types.layout(field.ty).align);
        index += 1;
    }
    bytes
}

/// Each local some block sets with a class constructor, in order.
fn constructed_locals(body: &Body) -> Vec<LocalId> {
    let mut out = Vec::new();
    for block in &body.blocks {
        if let Terminator::Call { func: FuncRef::Builtin { which: Builtin::ClassNew { .. }, .. }, dest, .. } = &block.terminator
            && dest.projection.is_empty()
            && dest.local != RETURN_LOCAL
            && !out.contains(&dest.local)
        {
            out.push(dest.local);
        }
    }
    out
}

/// Whether `place` is an alias's handle itself rather than a field through it: the local, or the
/// local read through a reference (a `self` taken as `ref mut`, as `drop`'s is).
fn whole(place: &Place, aliases: &HashSet<LocalId>) -> bool {
    aliases.contains(&place.local) && matches!(place.projection.as_slice(), [] | [Projection::Deref])
}

/// The alias whose handle `operand` reads whole.
fn reads_whole(operand: &Operand, aliases: &HashSet<LocalId>) -> Option<LocalId> {
    match operand {
        Operand::Copy(place) | Operand::Move(place) if whole(place, aliases) => Some(place.local),
        _ => None,
    }
}

/// The locals that ever hold the handle `start` holds: `start`, and every local set from an alias
/// whole (`M = L`, `M = L as Base`), to a fixpoint. Flow-insensitive, so a local that holds it on
/// any path is one.
fn aliases_of(body: &Body, start: LocalId) -> HashSet<LocalId> {
    let mut aliases = HashSet::from([start]);
    let mut changed = true;
    while changed {
        changed = false;
        for block in &body.blocks {
            for stmt in &block.stmts {
                let StmtKind::Assign { place, rvalue } = &stmt.kind else { continue };
                if !place.projection.is_empty() || aliases.contains(&place.local) {
                    continue;
                }
                let copied = match rvalue {
                    Rvalue::Use(operand) => reads_whole(operand, &aliases).is_some(),
                    Rvalue::Cast { kind: CastKind::ClassUpcast | CastKind::ClassUpcastBorrowed, operand, .. } => {
                        reads_whole(operand, &aliases).is_some()
                    }
                    _ => false,
                };
                if copied {
                    aliases.insert(place.local);
                    changed = true;
                }
            }
        }
    }
    aliases
}

/// For each body, by symbol, whether each parameter's handle escapes it: a greatest fixpoint, every
/// parameter first taken not to escape, until no body's use of one says otherwise.
fn parameter_escapes(bodies: &[Body]) -> HashMap<String, Vec<bool>> {
    let mut escapes: HashMap<String, Vec<bool>> =
        bodies.iter().filter(|body| !body.is_extern_declaration && !body.is_abstract).map(|body| (body.symbol.clone(), vec![false; body.arg_count])).collect();
    let mut changed = true;
    while changed {
        changed = false;
        for body in bodies.iter().filter(|body| !body.is_extern_declaration && !body.is_abstract) {
            for param in 0..body.arg_count {
                if escapes[&body.symbol][param] {
                    continue;
                }
                let aliases = aliases_of(body, LocalId(param as u32 + 1));
                if escapes_body(body, &aliases, &escapes) {
                    escapes.get_mut(&body.symbol).expect("each body's entry")[param] = true;
                    changed = true;
                }
            }
        }
    }
    escapes
}

/// Whether any use of an alias lets the handle outlive `body` (the module's description).
fn escapes_body(body: &Body, aliases: &HashSet<LocalId>, escapes: &HashMap<String, Vec<bool>>) -> bool {
    let escaping_operand = |operand: &Operand| reads_whole(operand, aliases).is_some();
    for block in &body.blocks {
        for stmt in &block.stmts {
            match &stmt.kind {
                StmtKind::Assign { place, rvalue } => {
                    match rvalue {
                        // A copy into a local makes an alias; into anything else, or the result, it escapes.
                        Rvalue::Use(operand) => {
                            if escaping_operand(operand) && (!place.projection.is_empty() || place.local == RETURN_LOCAL) {
                                return true;
                            }
                        }
                        Rvalue::Cast { kind, operand, .. } => {
                            if escaping_operand(operand)
                                && (!matches!(kind, CastKind::ClassUpcast | CastKind::ClassUpcastBorrowed)
                                    || !place.projection.is_empty()
                                    || place.local == RETURN_LOCAL)
                            {
                                return true;
                            }
                        }
                        Rvalue::Aggregate { operands, .. } => {
                            if operands.iter().any(escaping_operand) {
                                return true;
                            }
                        }
                        Rvalue::Repeat { value, .. } => {
                            if escaping_operand(value) {
                                return true;
                            }
                        }
                        // The address of the local holding the handle: whoever is given it can copy
                        // the handle out. Of a field through it, a borrow of the object's memory,
                        // which the borrow checker keeps within the handle's life.
                        Rvalue::Ref { place: borrowed, .. } => {
                            if whole(borrowed, aliases) {
                                return true;
                            }
                        }
                        Rvalue::BinaryOp { .. } | Rvalue::UnaryOp { .. } | Rvalue::Discriminant(_) => {}
                    }
                    // Setting an alias from anything else replaces its handle; nothing escapes.
                }
                StmtKind::CheckedBinaryOp { .. }
                | StmtKind::BeginAccess { .. }
                | StmtKind::BeginAccessTransfer { .. }
                | StmtKind::EndAccess { .. }
                | StmtKind::EndAccessTransfer { .. }
                | StmtKind::LendAccess { .. }
                | StmtKind::StorageLive(_)
                | StmtKind::StorageDead(_)
                | StmtKind::Drop { .. }
                | StmtKind::Nop => {}
            }
        }
        if let Terminator::Call { func, args, .. } = &block.terminator {
            for (index, arg) in args.iter().enumerate() {
                if !escaping_operand(arg) {
                    continue;
                }
                let kept = match func {
                    FuncRef::Direct { symbol, .. } => escapes.get(symbol).and_then(|params| params.get(index)).is_none_or(|escapes| *escapes),
                    _ => true,
                };
                if kept {
                    return true;
                }
            }
        }
        if let Terminator::Return = block.terminator
            && aliases.contains(&RETURN_LOCAL)
        {
            return true;
        }
    }
    false
}

/// Whether, where the constructor makes the frame's object again, no alias holding the one made
/// before is still to be used: a backward liveness over the aliases. A counted alias is dropped (a
/// use) before its object is gone, so while it holds the last object it is live; an uncounted copy
/// (a borrowed parameter inlined, say) holds no count and only must not be read again.
fn released_where_needed(body: &Body, aliases: &HashSet<LocalId>, local: LocalId) -> bool {
    let count = body.blocks.len();
    let mut predecessors: Vec<Vec<usize>> = vec![Vec::new(); count];
    for (index, block) in body.blocks.iter().enumerate() {
        for next in successors(&block.terminator) {
            predecessors[next.0 as usize].push(index);
        }
    }
    // The aliases live where each block starts.
    let mut live_in: Vec<HashSet<LocalId>> = vec![HashSet::new(); count];
    let mut work: Vec<usize> = (0..count).collect();
    while let Some(block) = work.pop() {
        let mut live: HashSet<LocalId> = successors(&body.blocks[block].terminator)
            .iter()
            .flat_map(|next| live_in[next.0 as usize].iter().copied())
            .collect();
        terminator_liveness(&body.blocks[block].terminator, aliases, &mut live);
        for stmt in body.blocks[block].stmts.iter().rev() {
            stmt_liveness(&stmt.kind, aliases, &mut live);
        }
        if live != live_in[block] {
            live_in[block] = live;
            work.extend(predecessors[block].iter().copied());
        }
    }
    body.blocks.iter().all(|block| match &block.terminator {
        Terminator::Call { func: FuncRef::Builtin { which: Builtin::ClassNew { .. }, .. }, dest, next, .. }
            if dest.projection.is_empty() && dest.local == local =>
        {
            // After the constructor: what the next block needs, less the new handle itself.
            live_in[next.0 as usize].iter().all(|alias| *alias == local)
        }
        _ => true,
    })
}

/// The aliases `place` reads: its base local, whole or through a projection.
fn read_place(place: &Place, aliases: &HashSet<LocalId>, live: &mut HashSet<LocalId>) {
    if aliases.contains(&place.local) {
        live.insert(place.local);
    }
}

fn read_operand(operand: &Operand, aliases: &HashSet<LocalId>, live: &mut HashSet<LocalId>) {
    if let Operand::Copy(place) | Operand::Move(place) = operand {
        read_place(place, aliases, live);
    }
}

/// Liveness through one statement, backwards: a whole write kills, every read (a write through a
/// projection reads its base) makes live.
fn stmt_liveness(kind: &StmtKind, aliases: &HashSet<LocalId>, live: &mut HashSet<LocalId>) {
    match kind {
        StmtKind::Assign { place, rvalue } => {
            if place.projection.is_empty() {
                live.remove(&place.local);
            } else {
                read_place(place, aliases, live);
            }
            match rvalue {
                Rvalue::Use(operand) | Rvalue::Cast { operand, .. } | Rvalue::UnaryOp { operand, .. } => {
                    read_operand(operand, aliases, live)
                }
                Rvalue::BinaryOp { lhs, rhs, .. } => {
                    read_operand(lhs, aliases, live);
                    read_operand(rhs, aliases, live);
                }
                Rvalue::Aggregate { operands, .. } => operands.iter().for_each(|operand| read_operand(operand, aliases, live)),
                Rvalue::Repeat { value, .. } => read_operand(value, aliases, live),
                Rvalue::Discriminant(place) | Rvalue::Ref { place, .. } => read_place(place, aliases, live),
            }
        }
        StmtKind::CheckedBinaryOp { dest, overflow, lhs, rhs, .. } => {
            for written in [dest, overflow] {
                if written.projection.is_empty() {
                    live.remove(&written.local);
                } else {
                    read_place(written, aliases, live);
                }
            }
            read_operand(lhs, aliases, live);
            read_operand(rhs, aliases, live);
        }
        StmtKind::BeginAccess { place, .. }
        | StmtKind::BeginAccessTransfer { place, .. }
        | StmtKind::EndAccess { place, .. }
        | StmtKind::EndAccessTransfer { place, .. }
        | StmtKind::LendAccess { place, .. }
        | StmtKind::Drop { place, .. } => read_place(place, aliases, live),
        StmtKind::StorageLive(_) | StmtKind::StorageDead(_) | StmtKind::Nop => {}
    }
}

fn terminator_liveness(terminator: &Terminator, aliases: &HashSet<LocalId>, live: &mut HashSet<LocalId>) {
    match terminator {
        Terminator::Call { args, dest, .. } => {
            if dest.projection.is_empty() {
                live.remove(&dest.local);
            } else {
                read_place(dest, aliases, live);
            }
            for arg in args {
                read_operand(arg, aliases, live);
            }
        }
        Terminator::SwitchInt { discr, .. } => read_operand(discr, aliases, live),
        Terminator::Assert { cond, .. } => read_operand(cond, aliases, live),
        Terminator::Goto(_) | Terminator::Return | Terminator::Unreachable => {}
    }
}

fn successors(terminator: &Terminator) -> Vec<BasicBlockId> {
    match terminator {
        Terminator::Goto(next) => vec![*next],
        Terminator::SwitchInt { targets, otherwise, .. } => targets.iter().map(|(_, target)| *target).chain([*otherwise]).collect(),
        Terminator::Call { next, .. } | Terminator::Assert { next, .. } => vec![*next],
        Terminator::Return | Terminator::Unreachable => Vec::new(),
    }
}
