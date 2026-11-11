//! Moves and drop elaboration (Part XVIII §4.9, `[OWN-2]`, `[OWN-3]`,
//! `[DRP-2]`).
//!
//! A value's owner drops it when the owner's scope ends — unless the value was
//! moved out first, in which case the new owner drops it and the old one must
//! not. Which of the two applies is not always known at compile time:
//!
//! ```ember
//! xs = Array()
//! if c:
//!     consume(xs)     # moved on this path
//! # dropped here, but only if `c` was false
//! ```
//!
//! `[OWN-3]` settles that with a **drop flag**: a hidden `bool` per local that
//! is conditionally moved, set when the value is put there and cleared when it
//! is moved out. A conditional move is never an error.
//!
//! This pass runs after lowering, over one body:
//!
//! 1. a forward dataflow computing live, moved and partial states per move path;
//! 2. whole and projected uses checked independently (`E3040`/`E3042`);
//! 3. drops of moved paths removed and conditional paths given flags;
//! 4. the flags' assignments written in beside the moves.

use std::collections::{BTreeMap, BTreeSet};

use ember_diag::{Diagnostic, Replacement, Sink, codes};
use ember_mir::{
    Body, FuncRef, LocalDecl, LocalId, LocalKind, Operand, Place, Projection, Rvalue, Stmt, StmtKind,
    Terminator,
};
use ember_span::Span;
use ember_types::{StructId, Ty, TypeTable, TyKind};

/// Possible states of one move path at a program point.
///
/// This is a three-bit powerset lattice rather than a three-variant enum. A
/// control-flow join can contain any combination of live, wholly moved and
/// partially moved states, and retaining the `PARTIAL` bit is what lets the
/// diagnostic distinguish `[EXP-6]`/`E3042` from an ordinary conditional
/// whole-value move.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
struct Owned(u8);

/// ODR-281 — the states of a body's move paths at one point, two to a byte (an `Owned` is three
/// bits): the elaboration holds one for each block, so a large body's took a byte per path per
/// block (a 3,200-call `main`'s, 430 MB). The join is the bytes' or, as `Owned::join` is each one's.
#[derive(Clone, PartialEq, Eq, Debug)]
struct States(Vec<u8>);

impl States {
    fn get(&self, node: usize) -> Owned {
        let byte = self.0[node / 2];
        Owned(if node % 2 == 0 { byte & 0x0F } else { byte >> 4 })
    }

    fn set(&mut self, node: usize, value: Owned) {
        let byte = &mut self.0[node / 2];
        *byte = if node % 2 == 0 { (*byte & 0xF0) | value.0 } else { (*byte & 0x0F) | (value.0 << 4) };
    }

    fn join(&self, other: &States) -> States {
        States(self.0.iter().zip(&other.0).map(|(a, b)| a | b).collect())
    }
}

impl Owned {
    const LIVE: Owned = Owned(1);
    const MOVED: Owned = Owned(2);
    const PARTIAL: Owned = Owned(4);

    /// The join of two states: their union, as sets of what may be (`States::join` joins two
    /// points' states a byte at a time, two paths in each).
    #[cfg(test)]
    fn join(self, other: Owned) -> Owned {
        Owned(self.0 | other.0)
    }

    fn contains(self, other: Owned) -> bool {
        self.0 & other.0 != 0
    }

    fn is_live(self) -> bool {
        self == Owned::LIVE
    }

    fn is_moved(self) -> bool {
        self == Owned::MOVED
    }

    fn is_partial(self) -> bool {
        self.contains(Owned::PARTIAL)
    }

    fn may_be_live(self) -> bool {
        self.contains(Owned::LIVE)
    }
}

/// A tree of places whose movedness can differ independently.
///
/// Every local has a root. Plain structs and tuples have child paths; types
/// with their own destructor, arrays and enums stay atomic because `[EXP-6]`
/// does not permit taking an owning element out of them. Keeping internal
/// aggregate nodes is important: moving the only field of a one-field struct
/// still leaves the *struct* partially moved rather than turning it into an
/// ordinary whole-value move.
struct MovePaths {
    places: Vec<Place>,
    parents: Vec<Option<usize>>,
    children: Vec<Vec<usize>>,
    /// Whether a path is a class handle: a write to a field behind it goes through the handle,
    /// which it reads and leaves as it was (D-698).
    handles: Vec<bool>,
    /// Per local, its root path.
    roots: Vec<usize>,
}

impl MovePaths {
    fn build(body: &Body, types: &TypeTable) -> MovePaths {
        let mut paths = MovePaths {
            places: Vec::new(),
            parents: Vec::new(),
            children: Vec::new(),
            handles: Vec::new(),
            roots: Vec::with_capacity(body.locals.len()),
        };
        for (index, local) in body.locals.iter().enumerate() {
            let root = paths.add(Place::local(LocalId(index as u32)), local.ty, None, types);
            paths.roots.push(root);
        }
        paths
    }

    fn add(
        &mut self,
        place: Place,
        ty: Ty,
        parent: Option<usize>,
        types: &TypeTable,
    ) -> usize {
        let index = self.places.len();
        self.places.push(place.clone());
        self.parents.push(parent);
        self.children.push(Vec::new());
        self.handles.push(matches!(types.kind(ty), TyKind::Class(_) | TyKind::ClassInterface(_)));

        let fields: Vec<Ty> = match types.kind(ty) {
            TyKind::Struct(id)
                if !types.struct_def(*id).has_drop && types.struct_def(*id).drops_fields =>
            {
                types.struct_def(*id).fields.iter().map(|field| field.ty).collect()
            }
            TyKind::Tuple(items) => items.clone(),
            _ => Vec::new(),
        };
        for (field, field_ty) in fields.into_iter().enumerate() {
            let child = self.add(place.clone().field(field), field_ty, Some(index), types);
            self.children[index].push(child);
        }
        index
    }

    fn initial(&self, body: &Body) -> States {
        let mut state = States(vec![0; self.places.len().div_ceil(2)]);
        for (node, place) in self.places.iter().enumerate() {
            state.set(node, match body.local(place.local).kind {
                LocalKind::Arg | LocalKind::Return => Owned::LIVE,
                _ => Owned::MOVED,
            });
        }
        state
    }

    /// The exact node, or the deepest atomic ancestor when a projection goes
    /// inside a type this pass deliberately treats as indivisible. D-811 — walked down from the
    /// local's root, field by field (a node's children are its fields, in order), not found by
    /// comparing every path of the function.
    fn node(&self, place: &Place) -> usize {
        let mut node = *self.roots.get(place.local.0 as usize).expect("every MIR local has a move-path root");
        for projection in &place.projection {
            let Projection::Field(field) = projection else { break };
            match self.children[node].get(*field) {
                Some(&child) => node = child,
                None => break,
            }
        }
        node
    }

    /// The class handle a write to `place` goes through, when `place` is a field of the object
    /// a handle names: `p.x = 1` reads `p` and gives it no value.
    fn through_handle(&self, place: &Place) -> Option<&Place> {
        let node = self.node(place);
        (self.handles[node] && self.places[node].projection.len() < place.projection.len()).then(|| &self.places[node])
    }

    fn state(&self, place: &Place, state: &States) -> Owned {
        state.get(self.node(place))
    }

    fn move_out(&self, place: &Place, state: &mut States) {
        // A field behind a class handle is the object's, never the handle's: dropping the old
        // value of `p.next` before a write leaves `p` as it was (D-698).
        if self.through_handle(place).is_some() {
            return;
        }
        let node = self.node(place);
        self.set_subtree(node, Owned::MOVED, state);
        self.recompute_ancestors(node, state);
    }

    fn initialise(&self, place: &Place, state: &mut States) {
        let node = self.node(place);
        if self.through_handle(place).is_some() {
            return;
        }
        self.set_subtree(node, Owned::LIVE, state);
        self.recompute_ancestors(node, state);
    }

    fn set_subtree(&self, node: usize, value: Owned, state: &mut States) {
        state.set(node, value);
        for &child in &self.children[node] {
            self.set_subtree(child, value, state);
        }
    }

    fn recompute_ancestors(&self, node: usize, state: &mut States) {
        let mut parent = self.parents[node];
        while let Some(index) = parent {
            // This ancestor was not itself moved or assigned. It is usable as
            // a whole only in combinations where every direct child is live;
            // every other possible combination is a partial value.
            let children = &self.children[index];
            let all_may_live = children.iter().all(|child| state.get(*child).may_be_live());
            let all_are_live = children.iter().all(|child| state.get(*child).is_live());
            let mut bits = 0;
            if all_may_live {
                bits |= Owned::LIVE.0;
            }
            if !all_are_live {
                bits |= Owned::PARTIAL.0;
            }
            state.set(index, Owned(bits));
            parent = self.parents[index];
        }
    }
}

fn place_prefix(prefix: &Place, place: &Place) -> bool {
    prefix.local == place.local
        && prefix.projection.len() <= place.projection.len()
        && prefix
            .projection
            .iter()
            .zip(&place.projection)
            .all(|(a, b)| a == b)
}

/// Rewrite one body so that every value is dropped exactly once, and report
/// every use of a moved value. Returns the number of errors.
pub fn elaborate(body: &mut Body, types: &TypeTable, sink: &mut Sink) -> usize {
    // `[DRP-5]` — a `drop` body may not move fields out of `mut self`.
    let mut errors = check_drop_moves(body, types, sink);
    // `[EXP-6]`/`[FN-1]` — moves out of borrowed places outside `drop` (D-041).
    errors += check_borrowed_moves(body, types, sink);
    let paths = MovePaths::build(body, types);
    let entry = paths.initial(body);
    let mut block_entry: Vec<Option<States>> = vec![None; body.blocks.len()];
    if body.blocks.is_empty() {
        return 0;
    }
    block_entry[0] = Some(entry);

    let mut worklist = vec![0usize];
    while let Some(index) = worklist.pop() {
        let Some(state) = block_entry[index].clone() else { continue };
        let exit = transfer(body, index, state, None, &paths);
        for successor in successors(body, index) {
            let merged = match &block_entry[successor] {
                Some(existing) => {
                    let joined = existing.join(&exit);
                    if joined == *existing {
                        continue;
                    }
                    joined
                }
                None => exit.clone(),
            };
            block_entry[successor] = Some(merged);
            worklist.push(successor);
        }
    }

    // A second pass reports and rewrites, once the entry states have settled.
    let cyclic = blocks_in_a_cycle(body);
    let mut reporter = Reporter {
        sink,
        types,
        errors: 0,
        reported: vec![false; body.locals.len()],
        in_loop: false,
        point: (0, 0),
    };
    let mut plan: BTreeMap<(usize, usize), DropAction> = BTreeMap::new();
    for (index, entry) in block_entry.iter().enumerate() {
        if let Some(state) = entry.clone() {
            let mut state = state;
            reporter.in_loop = cyclic[index];
            for (position, stmt) in body.blocks[index].stmts.iter().enumerate() {
                if let StmtKind::Drop { place, .. } = &stmt.kind {
                    plan.insert(
                        (index, position),
                        plan_drop(place, paths.state(place, &state), &state, &paths, body, types),
                    );
                }
                reporter.point = (index, position);
                step(stmt, &mut state, Some(&mut reporter), body, &paths);
            }
            reporter.point = (index, body.blocks[index].stmts.len());
            step_terminator(body, index, &mut state, Some(&mut reporter), &paths);
        }
    }
    errors += reporter.errors;

    // `[OWN-3]` — each move path that may be live at a drop needs its own
    // flag. A single whole-local flag cannot represent `pair.first` having
    // moved while `pair.second` remains live.
    let needs_flag: Vec<Place> = plan.values().flat_map(DropAction::flag_places).collect();
    let flags = create_flags(body, &needs_flag, types);

    // Apply the settled plan. A partial aggregate expands into drops of its
    // still-live owning leaves, in `[DRP-2]` order.
    for (block_index, block) in body.blocks.iter_mut().enumerate() {
        let mut rewritten = Vec::with_capacity(block.stmts.len());
        for (position, mut stmt) in std::mem::take(&mut block.stmts).into_iter().enumerate() {
            let Some(action) = plan.remove(&(block_index, position)) else {
                rewritten.push(stmt);
                continue;
            };
            match action {
                DropAction::Keep => rewritten.push(stmt),
                DropAction::Remove => {}
                DropAction::Guard(place) => {
                    if let StmtKind::Drop { flag, .. } = &mut stmt.kind {
                        *flag = flags.get(&place).copied();
                    }
                    rewritten.push(stmt);
                }
                DropAction::Expand(parts) => {
                    let scope_end = matches!(stmt.kind, StmtKind::Drop { scope_end: true, .. });
                    for (place, state) in parts {
                        if state.is_moved() || !state.may_be_live() {
                            continue;
                        }
                        let flag = if state.is_live() {
                            None
                        } else {
                            flags.get(&place).copied()
                        };
                        rewritten.push(Stmt::new(StmtKind::Drop { place, flag, scope_end }, stmt.span));
                    }
                }
            }
        }
        block.stmts = rewritten;
    }
    if !flags.is_empty() {
        write_flag_updates(body, &flags);
    }
    errors
}

#[derive(Debug)]
enum DropAction {
    Keep,
    Remove,
    /// Keep the original drop and guard it with this move path's flag.
    Guard(Place),
    /// Replace an aggregate drop with its independently owned drop units.
    Expand(Vec<(Place, Owned)>),
}

impl DropAction {
    fn flag_places(&self) -> Vec<Place> {
        match self {
            DropAction::Guard(place) => vec![place.clone()],
            DropAction::Expand(parts) => parts
                .iter()
                .filter(|(_, state)| state.may_be_live() && !state.is_live())
                .map(|(place, _)| place.clone())
                .collect(),
            DropAction::Keep | DropAction::Remove => Vec::new(),
        }
    }
}

fn plan_drop(
    place: &Place,
    owned: Owned,
    state: &States,
    paths: &MovePaths,
    body: &Body,
    types: &TypeTable,
) -> DropAction {
    if owned.is_live() {
        return DropAction::Keep;
    }
    if owned.is_moved() || !owned.may_be_live() && !owned.is_partial() {
        return DropAction::Remove;
    }
    if !owned.is_partial() {
        return DropAction::Guard(place.clone());
    }

    let mut units = Vec::new();
    drop_units(place.clone(), moved_place_ty(place, body, types), types, &mut units);
    DropAction::Expand(
        units
            .into_iter()
            .map(|unit| {
                let unit_state = paths.state(&unit, state);
                (unit, unit_state)
            })
            .collect(),
    )
}

/// Maximal independently droppable places, in the order `[DRP-2]` requires.
/// A plain aggregate has no destructor of its own, so it can be decomposed;
/// a type with a destructor must stay whole (and `[EXP-6]` forbids producing
/// a partial instance of it in the first place).
fn drop_units(place: Place, ty: Ty, types: &TypeTable, out: &mut Vec<Place>) {
    if !types.needs_drop(ty) {
        return;
    }
    match types.kind(ty) {
        TyKind::Struct(id)
            if !types.struct_def(*id).has_drop && types.struct_def(*id).drops_fields =>
        {
            let fields: Vec<Ty> =
                types.struct_def(*id).fields.iter().map(|field| field.ty).collect();
            for (index, field_ty) in fields.into_iter().enumerate().rev() {
                drop_units(place.clone().field(index), field_ty, types, out);
            }
        }
        TyKind::Tuple(items) => {
            for (index, item) in items.iter().copied().enumerate().rev() {
                drop_units(place.clone().field(index), item, types, out);
            }
        }
        _ => out.push(place),
    }
}

/// `[DRP-5]` — the `self` of a `drop` body, when this body is one.
///
/// A `drop` method's `mut self` is a `ref mut Owner` where `Owner` declares
/// `drop` (or the `Owner` itself for an owned-self spelling). Any other body,
/// including a free function named `drop`, is not a destructor.
fn drop_self(body: &Body, types: &TypeTable) -> Option<LocalId> {
    if body.name != "drop" {
        return None;
    }
    if body.arg_count != 1 {
        return None;
    }
    let decl = body.locals.get(1)?;
    if decl.kind != LocalKind::Arg {
        return None;
    }
    // A free function can also be named `drop` with one droppable parameter;
    // only a method receiver named `self` is a destructor (same test as
    // `receiver_is_a_view` in borrows.rs).
    if decl.name.as_deref() != Some("self") {
        return None;
    }
    let inner = match types.kind(decl.ty) {
        TyKind::Ref { inner, .. } => *inner,
        _ => decl.ty,
    };
    // `drop` is a special method name. It has no overloadable free-function
    // meaning here: a receiver named `self` of a nominal owner is exactly the
    // destructor body, including a class whose `has_drop` bit is being
    // materialized alongside its method bodies.
    matches!(types.kind(inner), TyKind::Struct(_) | TyKind::Enum(_) | TyKind::Class(_))
        .then_some(LocalId(1))
}

/// `[CLS-7a]` — reject a destructor that makes its own class handle outlive
/// the call.
///
/// A class handle is `Copy`, so the ordinary move checker intentionally does
/// not see `items.push(self)`. Publishing that copy is nevertheless forbidden:
/// it could retain the object while its deinitialisation is in progress. The
/// Compiler-known containers carry their publication fact directly in their
/// builtin operation. Reusing that fact lets this pass reject a visible
/// container/field/retained-token escape without treating an ordinary helper
/// call as an escape.
pub fn check_drop_self_escapes_all(bodies: &[Body], types: &TypeTable, sink: &mut Sink) -> usize {
    let owned_closure_environments: BTreeSet<StructId> = bodies
        .iter()
        .filter(|body| body.closure_captures_by_move)
        .filter_map(|body| body.closure_environment)
        .collect();
    bodies
        .iter()
        .filter(|body| !body.kept)
        .map(|body| check_drop_self_escapes(body, types, &owned_closure_environments, sink))
        .sum()
}

fn check_drop_self_escapes(
    body: &Body,
    types: &TypeTable,
    owned_closure_environments: &BTreeSet<StructId>,
    sink: &mut Sink,
) -> usize {
    let Some(self_local) = drop_self(body, types) else {
        return 0;
    };

    // A handle can first be copied into a local, then handed to the publishing
    // call. Track those aliases through the CFG; joining with union is
    // conservative and keeps the check valid across branches.
    let mut entries: Vec<Option<BTreeSet<LocalId>>> = vec![None; body.blocks.len()];
    if body.blocks.is_empty() {
        return 0;
    }
    entries[0] = Some(BTreeSet::new());
    let mut worklist = vec![0usize];
    while let Some(index) = worklist.pop() {
        let Some(entry) = entries[index].clone() else {
            continue;
        };
        let exit = self_handle_aliases_after(&body.blocks[index], entry, self_local);
        for successor in successors(body, index) {
            let merged = match &entries[successor] {
                Some(existing) => existing.union(&exit).copied().collect(),
                None => exit.clone(),
            };
            if entries[successor].as_ref() == Some(&merged) {
                continue;
            }
            entries[successor] = Some(merged);
            worklist.push(successor);
        }
    }

    let mut errors = 0;
    for (index, block) in body.blocks.iter().enumerate() {
        let Some(mut aliases) = entries[index].clone() else {
            continue;
        };
        for stmt in &block.stmts {
            if let StmtKind::Assign { place, rvalue } = &stmt.kind {
                if rvalue_copies_drop_self(rvalue, &aliases, self_local)
                    && ((!place.projection.is_empty()
                        && !is_in_self_storage(body, types, place, self_local))
                        || is_owned_closure_environment_local(
                            body,
                            types,
                            place.local,
                            owned_closure_environments,
                        ))
                {
                    report_drop_self_escape(sink, types, body.local(self_local).ty, stmt.span);
                    errors += 1;
                }
                update_self_handle_aliases(place, rvalue, &mut aliases, self_local);
            }
            if let StmtKind::StorageDead(local) = stmt.kind {
                aliases.remove(&local);
            }
        }

        if let Terminator::Call {
            func: FuncRef::Builtin { which, .. },
            args,
            ..
        } = &block.terminator
        {
            for argument in builtin_publishing_arguments(*which) {
                if args
                    .get(*argument)
                    .is_some_and(|argument| operand_is_drop_self(argument, &aliases, self_local))
                {
                    report_drop_self_escape(sink, types, body.local(self_local).ty, block.terminator_span);
                    errors += 1;
                }
            }
        }
    }
    errors
}

/// An `owned fn` environment owns every captured field and can escape its
/// creator. Its compiler-internal struct identity is the exact fact needed to
/// distinguish it from an ordinary aggregate local.
fn is_owned_closure_environment_local(
    body: &Body,
    types: &TypeTable,
    local: LocalId,
    owned_closure_environments: &BTreeSet<StructId>,
) -> bool {
    let Some(decl) = body.locals.get(local.0 as usize) else {
        return false;
    };
    matches!(types.kind(decl.ty), TyKind::Struct(id) if owned_closure_environments.contains(id))
}

/// The builtins that retain an argument in storage beyond the call itself.
/// Their ordinary signatures do not have a materialized callable body, but
/// their semantics are as precise as a verified `Publish` summary.
fn builtin_publishing_arguments(which: ember_mir::Builtin) -> &'static [usize] {
    match which {
        ember_mir::Builtin::ArrayPush
        | ember_mir::Builtin::CellSet
        | ember_mir::Builtin::CellReplace
        | ember_mir::Builtin::MaybeUninitWrite { .. }
        | ember_mir::Builtin::ArenaAlloc { .. }
        | ember_mir::Builtin::FixedArenaAlloc { .. }
        | ember_mir::Builtin::ScopedArenaAlloc { .. }
        | ember_mir::Builtin::ArenaArrayPush { .. } => &[1],
        ember_mir::Builtin::MaybeUninitWriteAt { .. }
        | ember_mir::Builtin::ArenaArrayInsert { .. } => &[2],
        ember_mir::Builtin::ArenaMapInsert { .. } => &[2, 3],
        ember_mir::Builtin::BoxNew { .. }
        | ember_mir::Builtin::SharedNew { .. }
        | ember_mir::Builtin::MemForget { .. } => &[0],
        _ => &[],
    }
}

fn self_handle_aliases_after(
    block: &ember_mir::BasicBlock,
    mut aliases: BTreeSet<LocalId>,
    self_local: LocalId,
) -> BTreeSet<LocalId> {
    for stmt in &block.stmts {
        match &stmt.kind {
            StmtKind::Assign { place, rvalue } => {
                update_self_handle_aliases(place, rvalue, &mut aliases, self_local);
            }
            StmtKind::StorageDead(local) => {
                aliases.remove(local);
            }
            _ => {}
        }
    }
    if let Terminator::Call { dest, .. } = &block.terminator
        && dest.projection.is_empty()
    {
        aliases.remove(&dest.local);
    }
    aliases
}

fn update_self_handle_aliases(
    place: &Place,
    rvalue: &Rvalue,
    aliases: &mut BTreeSet<LocalId>,
    self_local: LocalId,
) {
    if !place.projection.is_empty() {
        return;
    }
    aliases.remove(&place.local);
    if rvalue_copies_drop_self(rvalue, aliases, self_local) {
        aliases.insert(place.local);
    }
}

fn rvalue_copies_drop_self(
    rvalue: &Rvalue,
    aliases: &BTreeSet<LocalId>,
    self_local: LocalId,
) -> bool {
    match rvalue {
        Rvalue::Use(operand) | Rvalue::UnaryOp { operand, .. } | Rvalue::Cast { operand, .. } => {
            operand_is_drop_self(operand, aliases, self_local)
        }
        Rvalue::BinaryOp { lhs, rhs, .. } => {
            operand_is_drop_self(lhs, aliases, self_local)
                || operand_is_drop_self(rhs, aliases, self_local)
        }
        Rvalue::Aggregate { operands, .. } => operands
            .iter()
            .any(|operand| operand_is_drop_self(operand, aliases, self_local)),
        Rvalue::Repeat { value, .. } => operand_is_drop_self(value, aliases, self_local),
        Rvalue::Discriminant(_) | Rvalue::Ref { .. } => false,
    }
}

fn operand_is_drop_self(
    operand: &Operand,
    aliases: &BTreeSet<LocalId>,
    self_local: LocalId,
) -> bool {
    let (Operand::Copy(place) | Operand::Move(place)) = operand else {
        return false;
    };
    (place.local == self_local && place.projection == [Projection::Deref])
        || (aliases.contains(&place.local) && place.projection.is_empty())
}

/// `[OBJ-5]`, `[CLS-7a]` — the object being ended, named as the program names it, the statement as
/// written (its line), and a field to keep a copy of: "`drop` ends this `Player`; an owner made from it would
/// point at a freed object; keep a copy of what you need".
fn report_drop_self_escape(sink: &mut Sink, types: &TypeTable, self_ty: Ty, span: Span) {
    let inner = match types.kind(self_ty) {
        TyKind::Ref { inner, .. } => *inner,
        _ => self_ty,
    };
    let field = match types.kind(inner) {
        TyKind::Class(id) => types.class_def(*id).fields.first().map(|field| field.name),
        TyKind::Struct(id) => types.struct_def(*id).fields.first().map(|field| field.name),
        _ => None,
    };
    let line = ember_diag::line_mark(span);
    let help = match field {
        Some(field) => format!("on line {line}, hand over a copy of what you need where it has `this`: `copy of this.{field}`; or move the data out with `mem.take`"),
        None => format!("on line {line}, hand over a value of your own where it has `this`; or move the data out with `mem.take`"),
    };
    sink.emit_classified(
        Diagnostic::error(
            codes::E3016,
            span,
            format!(
                "`drop` ends this `{}`; an owner made from it would point at a freed object; keep a copy of what you need",
                types.display(inner)
            ),
        )
        .primary_label("published here")
        .note("a destructor may not publish a handle to the object being destroyed [CLS-7a]")
        .help(help),
    );
}

/// Whether a move leaves through `drop`'s `self`: rooted at it, past the borrow.
fn is_self_interior(place: &Place, self_local: LocalId) -> bool {
    place.local == self_local && !place.projection.is_empty()
}

/// A place in the storage of the object being destroyed (D-562): reached from
/// `self` without stepping into another object through a class handle,
/// reference, pointer or view held in one of its fields. `self.keeper.saved`
/// is `(*self).0.0`, a field of the `Keeper`, which outlives the drop.
fn is_in_self_storage(body: &Body, types: &TypeTable, place: &Place, self_local: LocalId) -> bool {
    if place.local != self_local || place.projection.is_empty() {
        return false;
    }
    let mut ty = body.local(self_local).ty;
    let mut variant = None;
    let mut at_self = true;
    for projection in &place.projection {
        let leaves = !at_self
            && match (projection, types.kind(ty)) {
                (Projection::Deref, _) | (Projection::Field(_), TyKind::Class(_)) => true,
                (
                    Projection::Index(_) | Projection::ConstIndex(_) | Projection::Column(_),
                    TyKind::Span { .. } | TyKind::Ptr { .. },
                ) => true,
                _ => false,
            };
        if leaves {
            return false;
        }
        at_self = at_self && matches!(projection, Projection::Deref);
        ty = crate::regions::project_type(types, ty, projection, &mut variant);
    }
    true
}

/// `[DRP-5]`, `[EXP-6]` — a `drop` body may not move out through `mut self`.
///
/// A field move (`E3010`) double-destroys: the moved value drops with its new
/// owner at the end of the `drop` body, then drops again as a field after
/// `drop` returns. A whole-value move through the borrow (`E3013`) is the same
/// shape through `[EXP-6]` and would also launder `self` into an owned local
/// for a second field move. Only `Move` matters: `Copy` fields stay `Copy`.
fn check_drop_moves(body: &Body, types: &TypeTable, sink: &mut Sink) -> usize {
    let Some(self_local) = drop_self(body, types) else {
        return 0;
    };
    let mut errors = 0;
    let mut report = |place: &Place, span: Span| {
        let field_move =
            place.projection.iter().any(|p| !matches!(p, Projection::Deref));
        if field_move {
            sink.emit_classified(
                Diagnostic::error(
                    codes::E3010,
                    span,
                    "cannot move a field out of `this` in `drop`",
                )
                .primary_label("moved out here")
                .note("a `drop` body may not take fields out of `this` [DRP-5]; the field is dropped again after `drop` returns [EXP-6]")
                .help("use `mem.take`, `mem.replace` or `swap` to leave a value in its place"),
            );
        } else {
            sink.emit_classified(
                Diagnostic::error(codes::E3013, span, "cannot move `this` out of `drop`")
                    .primary_label("moved out here")
                    .note("a `drop` body may not take `this` over [DRP-5]; the value is dropped again after `drop` returns [EXP-6]")
                    .help("borrow it instead of moving it; use `mem.take`, `mem.replace` or `swap` to leave a value in a field's place"),
            );
        }
        errors += 1;
    };
    let check_operand = |operand: &Operand, span: Span, report: &mut dyn FnMut(&Place, Span)| {
        if let Operand::Move(place) = operand {
            if is_self_interior(place, self_local) {
                report(place, span);
            }
        }
    };
    for block in &body.blocks {
        for stmt in &block.stmts {
            match &stmt.kind {
                StmtKind::Assign { rvalue, .. } => match rvalue {
                    Rvalue::Use(o) | Rvalue::UnaryOp { operand: o, .. } => {
                        check_operand(o, stmt.span, &mut report);
                    }
                    Rvalue::Cast { operand, .. } => {
                        check_operand(operand, stmt.span, &mut report);
                    }
                    Rvalue::BinaryOp { lhs, rhs, .. } => {
                        check_operand(lhs, stmt.span, &mut report);
                        check_operand(rhs, stmt.span, &mut report);
                    }
                    Rvalue::Aggregate { operands, .. } => {
                        for operand in operands {
                            check_operand(operand, stmt.span, &mut report);
                        }
                    }
                    Rvalue::Repeat { value, .. } => {
                        check_operand(value, stmt.span, &mut report);
                    }
                    Rvalue::Discriminant(_) | Rvalue::Ref { .. } => {}
                },
                StmtKind::CheckedBinaryOp { lhs, rhs, .. } => {
                    check_operand(lhs, stmt.span, &mut report);
                    check_operand(rhs, stmt.span, &mut report);
                }
                _ => {}
            }
        }
        let span = block.terminator_span;
        match &block.terminator {
            Terminator::Call { func, args, .. } => {
                if let ember_mir::FuncRef::Indirect { operand, .. } = func {
                    check_operand(operand, span, &mut report);
                }
                for arg in args {
                    check_operand(arg, span, &mut report);
                }
            }
            Terminator::SwitchInt { discr, .. } => {
                check_operand(discr, span, &mut report);
            }
            Terminator::Assert { cond, .. } => {
                check_operand(cond, span, &mut report);
            }
            Terminator::Goto(_) | Terminator::Return | Terminator::Unreachable => {}
        }
    }
    errors
}

/// The type a place denotes, following its projections (cf. `place_ty` in
/// `verify.rs`). A projection that does not apply leaves the type alone: the
/// type checker has already rejected such a program.
fn moved_place_ty(place: &Place, body: &Body, types: &TypeTable) -> Ty {
    let mut ty = body.local(place.local).ty;
    let mut variant = None;
    for projection in &place.projection {
        match (projection, types.kind(ty)) {
            (Projection::Downcast(v), TyKind::Enum(_)) => variant = Some(*v),
            (Projection::Field(i), TyKind::Enum(id)) => {
                let Some(v) = variant else { continue };
                let fields = &types.enum_def(*id).variants[v].fields;
                ty = fields.get(*i).map(|f| f.ty).unwrap_or(ty);
                variant = None;
            }
            (Projection::Field(i), TyKind::Struct(id)) => {
                let fields = &types.struct_def(*id).fields;
                ty = fields.get(*i).map(|f| f.ty).unwrap_or(ty);
            }
            (Projection::Field(i), TyKind::Tuple(items)) => {
                ty = items.get(*i).copied().unwrap_or(ty);
            }
            (
                Projection::Index(_) | Projection::ConstIndex(_) | Projection::Column(_),
                TyKind::Array { elem, .. } | TyKind::Vec { elem, .. } | TyKind::Span { elem, .. },
            ) => ty = *elem,
            (
                Projection::Index(_) | Projection::ConstIndex(_) | Projection::Column(_),
                TyKind::Ptr { inner, .. },
            ) => ty = *inner,
            (Projection::Deref, TyKind::Ref { inner, .. } | TyKind::Ptr { inner, .. }) => {
                ty = *inner
            }
            _ => {}
        }
    }
    ty
}

/// Whether a `Move` place goes through a safe reference (`ref`/`ref mut`).
///
/// `[EXP-6]` names `E3013` for moves out of a `ref`/`ref mut`. A deref of a
/// raw pointer is `unsafe` territory and is left to `[UNS-*]`: this answers
/// which one the first `Deref` in the chain goes through, walking the type
/// alongside the projections the way `place_ty` in `verify.rs` does.
/// Whether a place reaches a field of a class object through its handle.
fn through_class_field(place: &Place, body: &Body, types: &TypeTable) -> bool {
    let mut ty = body.local(place.local).ty;
    let mut variant = None;
    for projection in &place.projection {
        match (projection, types.kind(ty)) {
            (Projection::Field(_), TyKind::Class(_)) => return true,
            (Projection::Downcast(v), TyKind::Enum(_)) => variant = Some(*v),
            (Projection::Field(i), TyKind::Enum(id)) => {
                let Some(v) = variant else { continue };
                let fields = &types.enum_def(*id).variants[v].fields;
                ty = fields.get(*i).map(|f| f.ty).unwrap_or(ty);
                variant = None;
            }
            (Projection::Field(i), TyKind::Struct(id)) => {
                let fields = &types.struct_def(*id).fields;
                ty = fields.get(*i).map(|f| f.ty).unwrap_or(ty);
            }
            (Projection::Field(i), TyKind::Tuple(items)) => ty = items.get(*i).copied().unwrap_or(ty),
            (
                Projection::Index(_) | Projection::ConstIndex(_) | Projection::Column(_),
                TyKind::Array { elem, .. } | TyKind::Vec { elem, .. } | TyKind::Span { elem, .. },
            ) => ty = *elem,
            (Projection::Deref, TyKind::Ref { inner, .. } | TyKind::Ptr { inner, .. }) => ty = *inner,
            _ => {}
        }
    }
    false
}

/// The class type of the object whose field `place` reaches through its handle, if it does.
fn class_object_of_field(place: &Place, body: &Body, types: &TypeTable) -> Option<Ty> {
    if !through_class_field(place, body, types) {
        return None;
    }
    let mut ty = body.local(place.local).ty;
    for projection in &place.projection {
        match (projection, types.kind(ty)) {
            (Projection::Field(_), TyKind::Class(_)) => return Some(ty),
            (Projection::Field(i), TyKind::Struct(id)) => ty = types.struct_def(*id).fields.get(*i).map(|f| f.ty).unwrap_or(ty),
            (Projection::Field(i), TyKind::Tuple(items)) => ty = items.get(*i).copied().unwrap_or(ty),
            (Projection::Deref, TyKind::Ref { inner, .. } | TyKind::Ptr { inner, .. }) => ty = *inner,
            _ => {}
        }
    }
    None
}

/// A place as the program wrote it: `p.name`, `q.items`.
fn describe_place(place: &Place, body: &Body, types: &TypeTable) -> String {
    let mut text = body.local(place.local).name.clone().unwrap_or_else(|| format!("_{}", place.local.0));
    let mut ty = body.local(place.local).ty;
    for projection in &place.projection {
        match (projection, types.kind(ty)) {
            (Projection::Field(i), TyKind::Struct(id)) => {
                if let Some(field) = types.struct_def(*id).fields.get(*i) {
                    text.push('.');
                    text.push_str(field.name.as_str());
                    ty = field.ty;
                }
            }
            (Projection::Field(i), TyKind::Class(id)) => {
                if let Some(field) = types.class_def(*id).fields.get(*i) {
                    text.push('.');
                    text.push_str(field.name.as_str());
                    ty = field.ty;
                }
            }
            (Projection::Field(i), TyKind::Tuple(items)) => {
                text.push_str(&format!(".{i}"));
                ty = items.get(*i).copied().unwrap_or(ty);
            }
            (Projection::Deref, TyKind::Ref { inner, .. } | TyKind::Ptr { inner, .. }) => ty = *inner,
            _ => {}
        }
    }
    text
}

fn deref_through_ref(place: &Place, body: &Body, types: &TypeTable) -> bool {
    let mut ty = body.local(place.local).ty;
    let mut variant = None;
    for projection in &place.projection {
        match (projection, types.kind(ty)) {
            (Projection::Deref, TyKind::Ref { .. }) => return true,
            (Projection::Deref, TyKind::Ptr { .. }) => return false,
            // Unreachable in verified MIR: only references and raw pointers
            // deref. Fail closed — a deref in safe code is a borrow.
            (Projection::Deref, _) => return true,
            (Projection::Downcast(v), TyKind::Enum(_)) => variant = Some(*v),
            (Projection::Field(i), TyKind::Enum(id)) => {
                let Some(v) = variant else { continue };
                let fields = &types.enum_def(*id).variants[v].fields;
                ty = fields.get(*i).map(|f| f.ty).unwrap_or(ty);
                variant = None;
            }
            (Projection::Field(i), TyKind::Struct(id)) => {
                let fields = &types.struct_def(*id).fields;
                ty = fields.get(*i).map(|f| f.ty).unwrap_or(ty);
            }
            (Projection::Field(i), TyKind::Tuple(items)) => {
                ty = items.get(*i).copied().unwrap_or(ty);
            }
            (
                Projection::Index(_) | Projection::ConstIndex(_) | Projection::Column(_),
                TyKind::Array { elem, .. } | TyKind::Vec { elem, .. } | TyKind::Span { elem, .. },
            ) => ty = *elem,
            (
                Projection::Index(_) | Projection::ConstIndex(_) | Projection::Column(_),
                TyKind::Ptr { inner, .. },
            ) => ty = *inner,
            _ => {}
        }
    }
    false
}

/// `[EXP-6]`, `[FN-1]` — moves out of borrowed places, outside `drop` bodies.
///
/// Two shapes, one code (`E3013`, shape O2):
///
/// * a `Move` through a `Deref` — `x = r.inner` where `r: ref Outer`. The
///   referent still owns the value and drops it, so the new owner drops it a
///   second time. `[BRW-1]` forbids moving while shared borrows are live;
///   `[EXP-6]` forbids moving out of a reference unconditionally, which is
///   what is checked here rather than loan liveness.
/// * a `Move` out of a borrowed (`[FN-1]`) parameter — whole (`x = o`) or
///   field (`x = o.inner`). "The callee reads through a `ref A` … The
///   callee cannot mutate or move `a`." A borrowed parameter arrives as a
///   bitwise copy with no loan behind it, so no other analysis can see that
///   the caller still owns (and drops) the value; the move double-destroys
///   across the call boundary.
///
/// Only owning `Move`s matter: a `Copy` leaves the source intact, which is
/// why reads and copies through a shared borrow stay legal — and a move of a
/// value that owns nothing (`needs_drop` false) is a bitwise copy with no
/// second destruction to happen, so it stays legal too. Only `Ref` derefs: a
/// raw pointer is `unsafe` territory. A move of a `ref`/`ref mut` local itself
/// (no projection) transfers the borrow and stays legal, as does a move of a
/// borrowed view-typed parameter (`ref`, `Span`, `str` are `Copy`;
/// `ref mut`/`MutSpan[T]` reborrows travel as whole-local moves). `drop`'s
/// own `self` keeps `check_drop_moves`' specific messages and is skipped
/// here, so one move is never two errors.
/// Whether a value of this type holds a mutable borrow (`ref mut`, a
/// `MutSpan`), directly or in a field: copying it would make two.
fn holds_mutable_borrow(types: &TypeTable, ty: Ty, depth: usize) -> bool {
    if depth > 16 {
        return true;
    }
    match types.kind(ty) {
        TyKind::Ref { mutable, .. } => *mutable,
        TyKind::Span { mutable, .. } => *mutable,
        TyKind::Struct(id) => types.struct_def(*id).fields.iter().any(|field| holds_mutable_borrow(types, field.ty, depth + 1)),
        TyKind::Tuple(items) => items.iter().any(|&item| holds_mutable_borrow(types, item, depth + 1)),
        TyKind::Array { elem, .. } => holds_mutable_borrow(types, *elem, depth + 1),
        TyKind::Enum(id) => types
            .enum_def(*id)
            .variants
            .iter()
            .any(|variant| variant.fields.iter().any(|field| holds_mutable_borrow(types, field.ty, depth + 1))),
        _ => false,
    }
}

fn check_borrowed_moves(body: &Body, types: &TypeTable, sink: &mut Sink) -> usize {
    let drop_self_local = drop_self(body, types);
    let mut errors = 0;
    let mut report = |place: &Place, span: Span, binding: bool| {
        if drop_self_local == Some(place.local) {
            return;
        }
        // `[EXP-6]`, `[OWN-11]` — the object still holds the field, and every alias of the handle
        // can still reach it and would see the hole: `E3013` (`E3012` is folded into it).
        if let Some(object) = class_object_of_field(place, body, types) {
            let part = describe_place(place, body, types);
            let owner = body.local(place.local).name.clone().unwrap_or_else(|| "the object".to_string());
            let class = types.display(object);
            sink.emit_classified(
                Diagnostic::error(
                    codes::E3013,
                    span,
                    format!("`{owner}` is a `{class}` object, which other names may reach, so `{part}` cannot be taken out and leave a hole; take it and leave a value in its place: `mem.take(ref mut {part})`"),
                )
                .primary_label("taken out here")
                .note("the object keeps the field, and every handle to it can still reach it [OWN-11]")
                .help(format!("take it and leave a value in its place, `mem.take(ref mut {part})` or `mem.replace(ref mut {part}, …)`; or keep a copy: `copy of {part}`")),
            );
            errors += 1;
            return;
        }
        // Only ownership moves are rejected: a move of a value that owns
        // nothing is a bitwise copy the analyses already treat as one (D-041
        // is the double-destroy, and a `Panel` of `i32`s has nothing to
        // destroy twice — `retitle` in `tests/conformance/MOD-2/` moves a
        // borrowed one on every run). A `Copy` value moves only where it is
        // handed over (`[OWN-9]`: an `owns` argument, `b = owns n`), and is
        // checked as any other: a number is no more a view's to give than a
        // `String` is (`[EXP-6]`). A value holding a mutable borrow (a closure
        // that writes its captures, `[CLO-1]`) is not a copy either: two copies
        // would be two live mutable borrows of one place (D-422, the `E3050` a
        // direct copy gets).
        // `[OWN-11]`, `[EXP-6]` — a hand-over out of a name that does not own the value is `E3013`
        // whatever the type: the owner still holds it.
        let moved = moved_place_ty(place, body, types);
        // `[OWN-3]` — `b = a` of a value that owns nothing, out of a name that does not own it, is
        // what makes `b` a view (A1-4), not a hand-over; until that is built it keeps 0.9.10's
        // answer. A hand-over (an `owns` argument, a store) is `E3013` whatever the type.
        if binding && !types.needs_drop(moved) && !holds_mutable_borrow(types, moved, 0) && !types.is_copy(moved) {
            return;
        }
        // `[EXP-6]` (D-599) — an element of an array, `Array`, `Span` or `SoA` column stays in its
        // container, which drops it again: moving it out is `E3011`.
        if place.projection.iter().any(|p| matches!(p, Projection::Index(_) | Projection::ConstIndex(_))) {
            sink.emit_classified(
                Diagnostic::error(codes::E3011, span, "cannot move out of an element")
                    .primary_label("moved out here")
                    .note("the container still holds the element and drops it with the rest [EXP-6]")
                    .help("borrow it, `ref xs[i]` (`ref mut` to change it in place); take it out with `mem.take`, `mem.replace`, `swap`, `pop` or `remove`; or clone it"),
            );
            errors += 1;
            return;
        }
        let through_ref = place
            .projection
            .iter()
            .any(|p| matches!(p, Projection::Deref))
            && deref_through_ref(place, body, types);
        // A borrowed parameter passed by address (`[BRW-8]`) is a `ref T`
        // local, but moving out of it is moving out of the parameter.
        let by_address = matches!(
            types.kind(body.local(place.local).ty),
            TyKind::Ref { mutable: false, inner } if types.passed_by_address(*inner)
        );
        let borrowed = body.borrowed_params.contains(&place.local)
            && (by_address || !types.is_view(body.local(place.local).ty) || holds_mutable_borrow(types, moved, 0));
        if through_ref && !borrowed {
            // D-823 (B3) — shape O2's help: the catalogue's moves, then the copy that keeps the
            // owner's value as it is, the only one that applies where the owner is only lent.
            sink.emit_classified(
                Diagnostic::error(codes::E3013, span, "cannot move out of a reference")
                    .primary_label("moved out here")
                    .note("moving out of a `ref`/`ref mut` is forbidden [EXP-6]; the referent still owns the value and drops it")
                    .help("borrow it, `ref` (`ref mut` to change it where it is); move it out where its owner is held mutably, with `mem.take`, `mem.replace` or `swap`, or `pop`, `swap_remove`, `remove` or `drain` from a container; or copy it, `.clone()`, leaving the owner's value as it is"),
            );
            errors += 1;
        } else if borrowed {
            let note = if types.needs_drop(moved) || !holds_mutable_borrow(types, moved, 0) {
                "a borrowed parameter is owned by its caller [FN-1]; moving it out destroys the value twice — once here, once with the caller"
            } else {
                "a borrowed parameter is its caller's value [FN-1]; it holds a mutable borrow, and moving it out would leave the caller a second one [CLO-2]"
            };
            sink.emit_classified(
                Diagnostic::error(codes::E3013, span, "cannot move out of a borrowed parameter")
                    .primary_label("moved out here")
                    .note(note)
                    .help("the caller keeps its value: borrow it, `ref`, or copy what is moved, `.clone()`; or take the parameter `owns`, or `ref mut` and move it out with `mem.take`, `mem.replace` or `swap`, leaving a value behind"),
            );
            errors += 1;
        }
    };
    let check_operand = |operand: &Operand, span: Span, binding: bool, report: &mut dyn FnMut(&Place, Span, bool)| {
        if let Operand::Move(place) = operand {
            report(place, span, binding);
        }
    };
    for block in &body.blocks {
        for stmt in &block.stmts {
            match &stmt.kind {
                StmtKind::Assign { rvalue, .. } => match rvalue {
                    Rvalue::Use(o) => check_operand(o, stmt.span, true, &mut report),
                    Rvalue::UnaryOp { operand: o, .. } => check_operand(o, stmt.span, false, &mut report),
                    Rvalue::Cast { operand, .. } => {
                        check_operand(operand, stmt.span, false, &mut report);
                    }
                    Rvalue::BinaryOp { lhs, rhs, .. } => {
                        check_operand(lhs, stmt.span, false, &mut report);
                        check_operand(rhs, stmt.span, false, &mut report);
                    }
                    Rvalue::Aggregate { operands, .. } => {
                        for operand in operands {
                            check_operand(operand, stmt.span, false, &mut report);
                        }
                    }
                    Rvalue::Repeat { value, .. } => {
                        check_operand(value, stmt.span, false, &mut report);
                    }
                    Rvalue::Discriminant(_) | Rvalue::Ref { .. } => {}
                },
                StmtKind::CheckedBinaryOp { lhs, rhs, .. } => {
                    check_operand(lhs, stmt.span, false, &mut report);
                    check_operand(rhs, stmt.span, false, &mut report);
                }
                StmtKind::StorageLive(_)
                | StmtKind::StorageDead(_)
                | StmtKind::Drop { .. }
                | StmtKind::BeginAccess { .. }
                | StmtKind::BeginAccessTransfer { .. }
                | StmtKind::EndAccess { .. }
                | StmtKind::EndAccessTransfer { .. } | StmtKind::LendAccess { .. }
                | StmtKind::Nop => {}
            }
        }
        let span = block.terminator_span;
        match &block.terminator {
            Terminator::Call { func, args, .. } => {
                if let ember_mir::FuncRef::Indirect { operand, .. } = func {
                    check_operand(operand, span, false, &mut report);
                }
                for arg in args {
                    check_operand(arg, span, false, &mut report);
                }
            }
            Terminator::SwitchInt { discr, .. } => {
                check_operand(discr, span, false, &mut report);
            }
            Terminator::Assert { cond, .. } => {
                check_operand(cond, span, false, &mut report);
            }
            Terminator::Goto(_) | Terminator::Return | Terminator::Unreachable => {}
        }
    }
    errors
}

/// A `bool` local per conditionally moved path.
fn create_flags(
    body: &mut Body,
    wanted: &[Place],
    types: &TypeTable,
) -> BTreeMap<Place, LocalId> {
    let mut flags = BTreeMap::new();
    let bool_ty = find_bool(types).unwrap_or_else(|| body.locals[0].ty);
    for place in wanted {
        if flags.contains_key(place) {
            continue;
        }
        let flag = LocalId(body.locals.len() as u32);
        let span = body.local(place.local).span;
        let suffix = place
            .projection
            .iter()
            .map(|projection| match projection {
                Projection::Field(index) => format!("_field_{index}"),
                Projection::Index(local) => format!("_index_{}", local.0),
                Projection::ConstIndex(index) => format!("_index_{index}"),
                Projection::Deref => "_deref".to_string(),
                Projection::Downcast(variant) => format!("_variant_{variant}"),
                Projection::Column(index) => format!("_column_{index}"),
            })
            .collect::<String>();
        let name = body
            .local(place.local)
            .name
            .as_ref()
            .map(|name| format!("{name}{suffix}_live"));
        body.locals.push(LocalDecl {
            ty: bool_ty,
            kind: LocalKind::Temp,
            name,
            span,
        });
        flags.insert(place.clone(), flag);
    }
    flags
}

/// Set each flag where its path is stored, clear it where the path is moved
/// out, and initialise it from the root local's entry state.
fn write_flag_updates(body: &mut Body, flags: &BTreeMap<Place, LocalId>) {
    for block in body.blocks.iter_mut() {
        let mut rewritten: Vec<Stmt> = Vec::with_capacity(block.stmts.len());
        for stmt in std::mem::take(&mut block.stmts) {
            let span = stmt.span;
            let mut moved = Vec::new();
            let mut written = Vec::new();
            match &stmt.kind {
                StmtKind::Assign { place, rvalue } => {
                    moved_by_rvalue(rvalue, &mut moved);
                    written.push(place.clone());
                }
                StmtKind::CheckedBinaryOp { dest, overflow, lhs, rhs, .. } => {
                    moved_by_operand(lhs, &mut moved);
                    moved_by_operand(rhs, &mut moved);
                    written.push(dest.clone());
                    written.push(overflow.clone());
                }
                StmtKind::StorageLive(_)
                | StmtKind::StorageDead(_)
                | StmtKind::Drop { .. }
                | StmtKind::BeginAccess { .. }
                | StmtKind::BeginAccessTransfer { .. }
                | StmtKind::EndAccess { .. }
                | StmtKind::EndAccessTransfer { .. } | StmtKind::LendAccess { .. }
                | StmtKind::Nop => {}
            }
            rewritten.push(stmt);
            let clear: BTreeSet<LocalId> = flags
                .iter()
                .filter(|(path, _)| moved.iter().any(|place| paths_overlap(place, path)))
                .map(|(_, flag)| *flag)
                .collect();
            let set: BTreeSet<LocalId> = flags
                .iter()
                .filter(|(path, _)| written.iter().any(|place| place_prefix(place, path)))
                .map(|(_, flag)| *flag)
                .collect();
            for flag in clear {
                rewritten.push(assign_bool(flag, false, span));
            }
            for flag in set {
                rewritten.push(assign_bool(flag, true, span));
            }
        }

        // Moves into a call happen in the terminator. Clear their flags in
        // the final statements of this block, before control transfers. An
        // indirect callee can itself be the consumed value for `[CLO-6]`'s
        // `owned f: fn(...)` form, so it participates alongside arguments.
        if let Terminator::Call { func, args, .. } = &block.terminator {
            let mut moved = Vec::new();
            if let ember_mir::FuncRef::Indirect { operand: callee, .. } = func {
                moved_by_operand(callee, &mut moved);
            }
            for arg in args {
                moved_by_operand(arg, &mut moved);
            }
            let clear: BTreeSet<LocalId> = flags
                .iter()
                .filter(|(path, _)| moved.iter().any(|place| paths_overlap(place, path)))
                .map(|(_, flag)| *flag)
                .collect();
            for flag in clear {
                rewritten.push(assign_bool(flag, false, block.terminator_span));
            }
        }
        block.stmts = rewritten;
    }
    // A value can also arrive from a call, whose destination is written by a
    // terminator rather than by a statement — `xs = Array()` is one. The flag
    // is set at the top of the block control reaches next.
    let mut set_on_entry: Vec<(usize, LocalId, Span)> = Vec::new();
    for (index, block) in body.blocks.iter().enumerate() {
        if let Terminator::Call { dest, next, .. } = &block.terminator {
            for (path, flag) in flags {
                if place_prefix(dest, path) {
                    set_on_entry.push((next.0 as usize, *flag, block.terminator_span));
                }
            }
        }
        let _ = index;
    }
    for (block, flag, span) in set_on_entry {
        let mut stmts = vec![assign_bool(flag, true, span)];
        stmts.extend(std::mem::take(&mut body.blocks[block].stmts));
        body.blocks[block].stmts = stmts;
    }

    // User locals start empty; owned parameters arrive live. Initialising
    // every flag to false leaked a conditionally moved argument on its
    // not-moved path.
    let span = body.span;
    let mut prologue: Vec<Stmt> = flags
        .iter()
        .map(|(place, flag)| {
            let live = matches!(body.local(place.local).kind, LocalKind::Arg | LocalKind::Return);
            assign_bool(*flag, live, span)
        })
        .collect();
    prologue.extend(std::mem::take(&mut body.blocks[0].stmts));
    body.blocks[0].stmts = prologue;
}

fn paths_overlap(a: &Place, b: &Place) -> bool {
    place_prefix(a, b) || place_prefix(b, a)
}

fn assign_bool(flag: LocalId, value: bool, span: Span) -> Stmt {
    Stmt::new(
        StmtKind::Assign {
            place: Place::local(flag),
            rvalue: Rvalue::Use(Operand::Const(ember_mir::Const::Bool(value))),
        },
        span,
    )
}

fn find_bool(types: &TypeTable) -> Option<Ty> {
    types
        .all()
        .find(|(_, kind)| matches!(kind, ember_types::TyKind::Bool))
        .map(|(ty, _)| ty)
}

fn transfer(
    body: &Body,
    index: usize,
    mut state: States,
    mut reporter: Option<&mut Reporter>,
    paths: &MovePaths,
) -> States {
    for stmt in &body.blocks[index].stmts {
        step(stmt, &mut state, reporter.as_deref_mut(), body, paths);
    }
    step_terminator(body, index, &mut state, reporter, paths);
    state
}

/// A call's arguments are read by the terminator, not by a statement, so a use
/// of a moved value in an argument is only visible here. Both the fixpoint and
/// the reporting pass go through this, or the two would disagree about what
/// each block does.
fn step_terminator(
    body: &Body,
    index: usize,
    state: &mut States,
    reporter: Option<&mut Reporter>,
    paths: &MovePaths,
) {
    let span = body.blocks[index].terminator_span;
    let Terminator::Call { func, args, dest, .. } = &body.blocks[index].terminator else {
        return;
    };

    let callee = match func {
        ember_mir::FuncRef::Indirect { operand, .. } => Some(operand),
        _ => None,
    };
    let handle = paths.through_handle(dest).map(|handle| Operand::Copy(handle.clone()));
    consume_in_order(callee.into_iter().chain(args).chain(handle.as_ref()), state, reporter, span, body, paths);
    paths.initialise(dest, state);
}

/// `[OWN-3]` — an instruction's operands are evaluated in order, so a place
/// one operand moves is gone for every operand after it: `(s, s)`,
/// `two(u, u)`, `P(v, v)` and `[w, w]` each move one value twice (D-426).
/// Each operand's read is checked against the state the operands before it
/// left, and then its move is applied. The fixpoint and the reporting pass
/// both come through here, so the state they reach is the same.
fn consume_in_order<'o>(
    operands: impl IntoIterator<Item = &'o Operand>,
    state: &mut States,
    mut reporter: Option<&mut Reporter>,
    span: Span,
    body: &Body,
    paths: &MovePaths,
) {
    let mut moved_here: Vec<&Place> = Vec::new();
    for operand in operands {
        let place = match operand {
            Operand::Copy(place) | Operand::Move(place) => place,
            Operand::Const(_) => continue,
        };
        if let Some(reporter) = reporter.as_deref_mut() {
            let owned = paths.state(place, state);
            if !owned.is_live() {
                let again = moved_here.iter().any(|moved| paths_overlap(moved, place));
                reporter.report(place, owned, span, body, again);
            }
        }
        if let Operand::Move(place) = operand {
            paths.move_out(place, state);
            moved_here.push(place);
        }
    }
}

/// The operands of an rvalue in evaluation order.
fn operands_of(rvalue: &Rvalue) -> Vec<&Operand> {
    match rvalue {
        Rvalue::Use(o) | Rvalue::UnaryOp { operand: o, .. } => vec![o],
        Rvalue::Cast { operand, .. } => vec![operand],
        Rvalue::BinaryOp { lhs, rhs, .. } => vec![lhs, rhs],
        Rvalue::Aggregate { operands, .. } => operands.iter().collect(),
        Rvalue::Repeat { value, .. } => vec![value],
        Rvalue::Discriminant(_) | Rvalue::Ref { .. } => Vec::new(),
    }
}

fn step(
    stmt: &Stmt,
    state: &mut States,
    mut reporter: Option<&mut Reporter>,
    body: &Body,
    paths: &MovePaths,
) {
    match &stmt.kind {
        StmtKind::Assign { place, rvalue } => {
            // `[BRW-7]` — a borrow does not consume, so it has no operand,
            // but a reference into memory that was moved out of dangles.
            if let (Rvalue::Ref { place, .. }, Some(reporter)) = (rvalue, reporter.as_deref_mut()) {
                let owned = paths.state(place, state);
                if !owned.is_live() {
                    reporter.report_borrow(place, owned, stmt.span, body);
                }
            }
            // `[OWN-3]` — reading a moved value is `E3040`; so is writing a field through a moved
            // class handle (D-698).
            let handle = paths.through_handle(place).map(|handle| Operand::Copy(handle.clone()));
            consume_in_order(operands_of(rvalue).into_iter().chain(handle.as_ref()), state, reporter, stmt.span, body, paths);
            paths.initialise(place, state);
        }
        // `[TYP-8]` — checked arithmetic writes its result through its own
        // statement rather than an `Assign`, so without this arm the
        // destination was never marked live and the next read of it was
        // reported as a use after move. Every integer `a * b` bound to a
        // local went through here.
        StmtKind::CheckedBinaryOp { dest, overflow, lhs, rhs, .. } => {
            let handles: Vec<Operand> = [dest, overflow].into_iter().filter_map(|place| paths.through_handle(place))
                .map(|handle| Operand::Copy(handle.clone())).collect();
            consume_in_order([lhs, rhs].into_iter().chain(&handles), state, reporter, stmt.span, body, paths);
            for place in [dest, overflow] {
                paths.initialise(place, state);
            }
        }
        // A drop ends the value's life whatever it was.
        StmtKind::Drop { place, .. } => {
            paths.move_out(place, state);
        }
        StmtKind::StorageLive(local) => paths.move_out(&Place::local(*local), state),
        _ => {}
    }
}

/// Which places an rvalue moves out of. Projected places are retained: dropping
/// that information was D-042's double-destruction bug.
fn moved_by_rvalue(rvalue: &Rvalue, out: &mut Vec<Place>) {
    match rvalue {
        Rvalue::Use(o) | Rvalue::UnaryOp { operand: o, .. } => moved_by_operand(o, out),
        Rvalue::Cast { operand, .. } => moved_by_operand(operand, out),
        Rvalue::BinaryOp { lhs, rhs, .. } => {
            moved_by_operand(lhs, out);
            moved_by_operand(rhs, out);
        }
        Rvalue::Aggregate { operands, .. } => {
            operands.iter().for_each(|o| moved_by_operand(o, out))
        }
        Rvalue::Repeat { value, .. } => moved_by_operand(value, out),
        Rvalue::Discriminant(_) | Rvalue::Ref { .. } => {}
    }
}

fn moved_by_operand(operand: &Operand, out: &mut Vec<Place>) {
    if let Operand::Move(place) = operand {
        out.push(place.clone());
    }
}

/// Which blocks can reach themselves.
///
/// `[OWN-4]` is about a loop, and a loop in MIR is a block reachable from its
/// own successors — there is no `while` left by this point, only the graph.
/// Computed once per body: the bodies here are small enough that a reachability
/// walk per block is cheaper than the machinery to avoid one.
fn blocks_in_a_cycle(body: &Body) -> Vec<bool> {
    let count = body.blocks.len();
    let mut cyclic = vec![false; count];
    for start in 0..count {
        let mut seen = vec![false; count];
        let mut stack = successors(body, start);
        while let Some(next) = stack.pop() {
            if next == start {
                cyclic[start] = true;
                break;
            }
            if seen[next] {
                continue;
            }
            seen[next] = true;
            stack.extend(successors(body, next));
        }
    }
    cyclic
}

/// `[OWN-4]` — the moves of `local` that can reach `point` with no assignment
/// to it in between, earliest in the source first: which move left the value
/// gone for a use in the next iteration. Run only when `E3041` is reported.
fn moves_reaching(body: &Body, local: LocalId, point: (usize, usize)) -> Vec<Span> {
    let touches = |place: &Place| place.local == local;
    // What one statement (or the terminator, `None`) does to the set of
    // moves in flight: a move adds its span, an assignment to the whole
    // local clears it.
    let apply = |stmt: Option<&Stmt>, block: usize, set: &mut BTreeSet<(u32, u32)>, spans: &mut Vec<Span>| {
        let mut moved = Vec::new();
        let (span, assigned) = match stmt {
            Some(stmt) => {
                let assigned = match &stmt.kind {
                    StmtKind::Assign { place, rvalue } => {
                        moved_by_rvalue(rvalue, &mut moved);
                        (place.local == local && place.projection.is_empty()).then_some(())
                    }
                    _ => None,
                };
                (stmt.span, assigned)
            }
            None => {
                let block = &body.blocks[block];
                let Terminator::Call { func, args, dest, .. } = &block.terminator else { return };
                // Calling an `owned fn` consumes it, as `step_terminator` says.
                if let ember_mir::FuncRef::Indirect { operand: callee, .. } = func {
                    moved_by_operand(callee, &mut moved);
                }
                for arg in args {
                    moved_by_operand(arg, &mut moved);
                }
                (block.terminator_span, (dest.local == local && dest.projection.is_empty()).then_some(()))
            }
        };
        if moved.iter().any(touches) {
            let key = (span.start, span.end);
            if !spans.contains(&span) {
                spans.push(span);
            }
            set.clear();
            set.insert(key);
        }
        if assigned.is_some() {
            set.clear();
        }
    };
    let mut spans = Vec::new();
    let mut entry: Vec<Option<BTreeSet<(u32, u32)>>> = vec![None; body.blocks.len()];
    entry[0] = Some(BTreeSet::new());
    let mut worklist = vec![0usize];
    while let Some(index) = worklist.pop() {
        let Some(mut set) = entry[index].clone() else { continue };
        for stmt in &body.blocks[index].stmts {
            apply(Some(stmt), index, &mut set, &mut spans);
        }
        apply(None, index, &mut set, &mut spans);
        for successor in successors(body, index) {
            let merged = match &entry[successor] {
                Some(existing) if set.is_subset(existing) => continue,
                Some(existing) => existing.union(&set).copied().collect(),
                None => set.clone(),
            };
            entry[successor] = Some(merged);
            worklist.push(successor);
        }
    }
    let (block, index) = point;
    let Some(mut set) = entry[block].clone() else { return Vec::new() };
    for stmt in body.blocks[block].stmts.iter().take(index) {
        apply(Some(stmt), block, &mut set, &mut spans);
    }
    let mut reaching: Vec<Span> = spans.into_iter().filter(|span| set.contains(&(span.start, span.end))).collect();
    reaching.sort_by_key(|span| (span.start, span.end));
    reaching
}

fn successors(body: &Body, index: usize) -> Vec<usize> {
    match &body.blocks[index].terminator {
        Terminator::Goto(bb) => vec![bb.0 as usize],
        Terminator::Call { next, .. } | Terminator::Assert { next, .. } => vec![next.0 as usize],
        Terminator::SwitchInt { targets, otherwise, .. } => {
            let mut out: Vec<usize> = targets.iter().map(|(_, bb)| bb.0 as usize).collect();
            out.push(otherwise.0 as usize);
            out
        }
        Terminator::Return | Terminator::Unreachable => Vec::new(),
    }
}

struct Reporter<'a> {
    sink: &'a mut Sink,
    /// `[OWN-3]` — whether a moved value is a collection, whose use after the move gets the note
    /// about Python's aliasing (shape O1).
    types: &'a TypeTable,
    errors: usize,
    /// One diagnostic per local, so a moved value read in a loop is reported
    /// once rather than every iteration.
    reported: Vec<bool>,
    /// `[OWN-4]` — whether the block being reported on can reach itself.
    ///
    /// This is what tells a *move in a loop* from a *use after move*, and they
    /// are different rules with different codes and different advice. Set
    /// before each block rather than threaded through `step`, which does not
    /// otherwise need to know where it is.
    in_loop: bool,
    /// The statement being reported on, `(block, index)`; the terminator is
    /// index `stmts.len()`.
    point: (usize, usize),
}

impl Reporter<'_> {
    fn report(&mut self, place: &Place, state: Owned, span: Span, body: &Body, again: bool) {
        let local = place.local;
        let index = local.0 as usize;
        if self.reported[index] {
            return;
        }
        // A compiler temporary that is `Moved` is one this pass has not been
        // taught about; only user variables are reported.
        let decl = body.local(local);
        if decl.kind == LocalKind::Temp || decl.name.is_none() {
            return;
        }
        // A local no move reaches was never given a value on that path:
        // definite initialisation reports that read (`E3050`), and one error
        // is enough ([DIA-14]). A move by an earlier operand of this same
        // instruction reaches it, though no earlier statement moved it.
        if !again && !state.is_partial() && moves_reaching(body, local, self.point).is_empty() {
            return;
        }
        self.reported[index] = true;
        self.errors += 1;
        let name = decl.name.clone().unwrap_or_else(|| format!("_{}", local.0));
        if again && !state.is_partial() {
            // D-426 — `(s, s)`, `f(s, s)`: the second operand finds the value
            // the first one took.
            self.sink.emit_classified(
                Diagnostic::error(codes::E3040, span, format!("`{name}` is handed over twice in one expression"))
                    .primary_label("a part of this expression before it already handed the value over")
                    .note(concat!(
                        "the parts of a call, a tuple, a list or a constructor are evaluated left to ",
                        "right, and the first that takes the value gives it away [OWN-3]"
                    ))
                    .help(format!("to keep both, give one of the two a copy of your own: `copy of {name}`")),
            );
            return;
        }
        if state.is_partial() {
            // `[OWN-11]` (`E3042` is folded into `E3040`) — a part handed over on its own, then its
            // owner used whole before the part has a new value.
            let moved_at = moves_reaching(body, local, self.point).into_iter().find(|moved| *moved != span);
            let site = moved_at.and_then(|at| hand_over_site(body, self.types, local, at));
            let part = site.as_ref().and_then(|site| site.part.clone()).unwrap_or_else(|| name.clone());
            let message = match (&site, moved_at) {
                (Some(HandOver { dest: Some(dest), .. }), Some(at)) => format!("`{part}` was handed over to `{dest}` on line {}; `{name}` is used whole on line {}", ember_diag::line_mark(at), ember_diag::line_mark(span)),
                (_, Some(at)) => format!("`{part}` was handed over on line {}; `{name}` is used whole on line {}", ember_diag::line_mark(at), ember_diag::line_mark(span)),
                _ => format!("`{part}` was handed over; `{name}` is used whole on line {}", ember_diag::line_mark(span)),
            };
            let keep = match site.as_ref().map(|site| &site.keep) {
                Some(Keep::Call { receiver, callee }) => format!("`{receiver}.{callee}(owns copy of {part})`"),
                Some(Keep::Store { dest }) => format!("`{dest} = copy of {part}`"),
                _ => format!("`copy of {part}`"),
            };
            let mut diagnostic = Diagnostic::error(codes::E3040, span, format!("{message}; give it a new value first, or keep a copy: {keep}"))
                // `[DIA-7a]` shape O1, a part: a new value first, or a copy of your own.
                .help(format!("give `{part}` a new value before `{name}` is used whole (`{part} = …`), or keep a copy: {keep}"))
                .primary_label("used here")
                .note("a part handed over on its own leaves the rest of the value usable, but neither the part nor the whole until the part has a new value [OWN-11]");
            if let Some(at) = moved_at {
                diagnostic = diagnostic.secondary(at, "handed over here");
            }
            self.sink.emit_classified(diagnostic);
            return;
        }
        // `[OWN-4]` — "A loop body that hands over a value declared outside the loop
        // is `E3040`, unless every path gives it a value again before the next round".
        // Its message is about the next round rather than about this use (`E3041`,
        // which said so, is folded into `E3040`; `[DIA-7a]` files it under O1).
        //
        // The two are told apart by where the use is and what the analysis
        // knows. A move and a use in one iteration leaves the local definitely
        // `Moved`, which is O1. A move that reaches its own use round a back
        // edge leaves it `MaybeMoved` at the loop head, because the entry state
        // joins "not yet moved" with "moved last time" — so `MaybeMoved` inside
        // a cycle is exactly `[OWN-4]`'s shape.
        let in_loop = self.in_loop && !state.is_moved();
        let diagnostic = if in_loop {
            // The use is where the analysis notices; the move that left the
            // value gone for the next iteration is what to point at.
            let moved_at = moves_reaching(body, local, self.point).into_iter().find(|moved| *moved != span);
            // `[OWN-4]` (`E3041` is folded into `E3040`) — the value is gone in the next round.
            let at = moved_at.unwrap_or(span);
            let diagnostic = Diagnostic::error(
                codes::E3040,
                at,
                format!(
                    "`{name}` is handed over on line {} and the loop comes round again, so the next round finds it gone; give it a new value before the next round (`{name} = …`), declare it inside the loop, or hand over a copy of your own: `copy of {name}`",
                    ember_diag::line_mark(at)
                ),
            );
            let diagnostic = match moved_at {
                Some(_) => diagnostic.secondary(span, "and used here in the next round"),
                None => diagnostic,
            };
            let diagnostic = self.kept_by_copy(diagnostic, body, local, again.then_some(span));
            diagnostic
            .primary_label("handed over here, and the loop comes round again")
            .note(concat!(
                "the first round hands it over and the next finds it gone; a value ",
                "handed over in a loop must be given a value again before the next round [OWN-4]"
            ))
            .help(concat!(
                "declare it inside the loop, hand over a copy of your own for each round, or use ",
                "`mem.take` when replacing it on every pass"
            ))
        } else {
            let (message, help) = self.hand_over_message(body, local, &name, span, state.is_moved());
            // `[DIA-7a]` shape O1 — the message names the ways out (`[OWN-3]`), and the help is the
            // primary one, the line to write.
            let diagnostic = Diagnostic::error(codes::E3040, span, message)
                .help(help)
                .primary_label("used here")
                .note("handing a value over gives it away; the old owner cannot use it again [OWN-3]");
            let diagnostic = self.kept_by_copy(diagnostic, body, local, again.then_some(span));
            self.moved_from(diagnostic, body, local, span)
        };
        self.sink.emit_classified(diagnostic);
    }

    /// `[OWN-3]` — "`items` was handed over to `snapshot` on line 3 and is used on line 4; to keep
    /// both, write `snapshot = copy of items` (a copy of your own) or `snapshot = shares items`
    /// (two owners of one list)". `sure`: the value is gone on every path that reaches the use.
    fn hand_over_message(&self, body: &Body, local: LocalId, name: &str, span: Span, sure: bool) -> (String, String) {
        let moved_at = moves_reaching(body, local, self.point).into_iter().find(|moved| *moved != span);
        let site = moved_at.and_then(|at| hand_over_site(body, self.types, local, at));
        let was = if sure { "was" } else { "may have been" };
        let used = ember_diag::line_mark(span);
        // `[CLO-6]` — "calling `f` uses it up; a second call is `E3040`": a value of a `once fn` type,
        // or one whose own call handed it over.
        let once = matches!(self.types.kind(body.local(local).ty), TyKind::Fn { once: true, .. })
            || matches!(&site, Some(HandOver { keep: Keep::Called, .. }));
        if let (true, Some(at)) = (once, moved_at) {
            return (
                format!(
                    "`{name}` is a `once fn`: its call on line {} uses it up, and it is used again on line {used}; a `once fn` runs once (take `{name}: fn(…)` where it is called more than once)",
                    ember_diag::line_mark(at)
                ),
                format!("take `{name}: fn(…)` where it is called more than once"),
            );
        }
        let head = match (&site, moved_at) {
            (Some(HandOver { dest: Some(dest), .. }), Some(at)) => format!("`{name}` {was} handed over to `{dest}` on line {} and is used on line {used}", ember_diag::line_mark(at)),
            (_, Some(at)) => format!("`{name}` {was} handed over on line {} and is used on line {used}", ember_diag::line_mark(at)),
            _ => format!("`{name}` {was} handed over and is used on line {used}"),
        };
        let ty = body.local(local).ty;
        let (tail, help) = if self.types.is_copy(ty) {
            (format!("to keep it, write `owns copy of {name}`"), format!("write `owns copy of {name}` where it is handed over"))
        } else {
            match site.as_ref().map(|site| &site.keep) {
                Some(Keep::Store { dest }) => (
                    format!("to keep both, write `{dest} = copy of {name}` (a copy of your own) or `{dest} = shares {name}` (two owners of one value)"),
                    format!("write `{dest} = copy of {name}` (a copy of your own) or `{dest} = shares {name}` (two owners), or use `{name}` before it is handed over"),
                ),
                _ => (
                    format!("to keep both, hand over a copy of your own: `copy of {name}`"),
                    format!("hand over `copy of {name}` (a copy of your own), or use `{name}` before it is handed over"),
                ),
            }
        };
        (format!("{head}; {tail}"), help)
    }

    /// `[CLI-20]` (N30) — 0.9.10 copied a number given to an `owned` parameter; 0.9.15 hands it over
    /// (`[OWN-9]`). Where such a value is used again, `ember fmt --migrate` gives each call that
    /// handed it over a copy (`take(owns copy of n)`), so the program does what it did; no message
    /// changes. `instruction`: the use's own instruction, when an earlier operand of it moved the value.
    fn kept_by_copy(&self, diagnostic: Diagnostic, body: &Body, local: LocalId, instruction: Option<Span>) -> Diagnostic {
        let moves: Vec<Span> = moves_reaching(body, local, self.point).into_iter().chain(instruction).collect();
        let mut edits: Vec<(Span, Replacement)> = Vec::new();
        for (handed, at) in &body.handed_at {
            let within = moves.iter().any(|moved| moved.file == at.file && moved.start <= at.start && at.end <= moved.end);
            let insert = Span::new(at.file, at.start, at.start);
            if *handed == local && within && !edits.iter().any(|(span, _)| *span == insert) {
                edits.push((insert, Replacement::Text("copy of ".to_string())));
            }
        }
        if edits.is_empty() {
            return diagnostic;
        }
        diagnostic.migrate_edits("a copy where 0.9.10 copied the number (`owns copy of`)", edits)
    }

    /// `[OWN-3]` — a use after a move is labelled at the move as well; and a collection's, with the
    /// note that assignment moves it where Python's would alias it (shape O1).
    fn moved_from(&self, mut diagnostic: Diagnostic, body: &Body, local: LocalId, span: Span) -> Diagnostic {
        if let Some(moved) = moves_reaching(body, local, self.point).into_iter().find(|moved| *moved != span) {
            diagnostic = diagnostic.secondary(moved, "moved here");
        }
        // `[DIA-7a]` shape O1 — "for a list, a note that `=` hands a list over where Python's aliases it".
        if is_collection(self.types, body.local(local).ty) {
            diagnostic = diagnostic.note(
                "`=` hands a list, a map or a set over; in Python both names would point at one list (Appendix E)",
            );
        }
        diagnostic
    }

    /// `[OWN-3]`, `[BRW-7]` (ODR-114) — a borrow a move may have reached is a use after the move,
    /// `E3040`, shape O1; a place never assigned on some path is definite initialisation's `E3050`.
    fn report_borrow(&mut self, place: &Place, state: Owned, span: Span, body: &Body) {
        let local = place.local;
        let index = local.0 as usize;
        if self.reported[index] {
            return;
        }
        let decl = body.local(local);
        if decl.kind == LocalKind::Temp || decl.name.is_none() {
            return;
        }
        // As for a use: a local no move reaches was never given a value, and
        // definite initialisation reports that borrow (`E3050`) ([DIA-14]).
        if !state.is_partial() && moves_reaching(body, local, self.point).is_empty() {
            return;
        }
        self.reported[index] = true;
        self.errors += 1;
        let name = decl.name.clone().unwrap_or_else(|| format!("_{}", local.0));
        let (message, help) = self.hand_over_message(body, local, &name, span, state.is_moved());
        let note = if state.is_moved() {
            "the view would point at memory whose owner gave it away"
        } else {
            "on at least one path reaching this line the value is gone"
        };
        let diagnostic = Diagnostic::error(codes::E3040, span, message)
            .help(help)
            .primary_label("used here")
            .note(note);
        let diagnostic = self.kept_by_copy(diagnostic, body, local, None);
        let diagnostic = self.moved_from(diagnostic, body, local, span);
        self.sink.emit_classified(diagnostic);
    }
}

/// A name the program wrote: it begins with a letter or one `_` (`__it`, a `for` loop's iterator, and
/// `$receiver` are the compiler's own).
fn written_name(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some('_') => chars.next() != Some('_'),
        Some(first) => first.is_alphabetic(),
        None => false,
    }
}

/// Where a value was handed over: to what, and how the program keeps a copy there.
struct HandOver {
    /// The place or function it went to: `snapshot` in `snapshot = items`, `names` in
    /// `names.push(owns p.name)`.
    dest: Option<String>,
    /// For a part handed over on its own, the part as written (`p.name`).
    part: Option<String>,
    keep: Keep,
}

enum Keep {
    Store { dest: String },
    Call { receiver: String, callee: String },
    /// `[CLO-6]` — the value is a `once fn`, used up by its own call.
    Called,
    Other,
}

/// The statement at `at` that handed `local` (or a part of it) over, as `[OWN-3]` and `[OWN-11]`
/// name it.
fn hand_over_site(body: &Body, types: &TypeTable, local: LocalId, at: Span) -> Option<HandOver> {
    // A function by the name the program wrote (`take`, not `em_4take`).
    // A method of `Option` or `Result` the checker routes to `std.core` is the method written
    // (`map`, not `option_map`).
    let short = |symbol: &str| {
        let demangled = ember_branding::demangle(symbol);
        if let Some(found) = &demangled
            && let Some(method) = ember_hir::routed_method(&found.components.join("."))
        {
            return method.to_string();
        }
        demangled
            .and_then(|found| found.components.last().cloned())
            .unwrap_or_else(|| symbol.rsplit(['.', ':']).next().unwrap_or(symbol).to_string())
    };
    // A place by the name the program wrote: not a temporary, nor a name the compiler declares
    // itself (`__it`, a `for` loop's iterator).
    let named = |place: &Place| {
        let decl = body.local(place.local);
        (decl.kind != LocalKind::Temp).then(|| decl.name.clone()).flatten().filter(|name| written_name(name))
    };
    for block in &body.blocks {
        for stmt in &block.stmts {
            if stmt.span != at {
                continue;
            }
            if let StmtKind::Assign { place, rvalue } = &stmt.kind {
                let mut moved = Vec::new();
                moved_by_rvalue(rvalue, &mut moved);
                if let Some(taken) = moved.iter().find(|moved| moved.local == local) {
                    // `[CLO-6]` — taken into the temporary the block's own call calls: the call uses it up.
                    // A callable parameter is made for each function given (`[CLO-3]`): its value is
                    // that function's item, and the call a direct call of it.
                    let item = match types.kind(body.local(local).ty) {
                        TyKind::FnItem { symbol, .. } => Some(symbol.as_str()),
                        _ => None,
                    };
                    let calls_it = match &block.terminator {
                        Terminator::Call { func: FuncRef::Indirect { operand: Operand::Move(callee) | Operand::Copy(callee), .. }, .. } => callee.local == place.local,
                        Terminator::Call { func: FuncRef::Direct { symbol, .. }, .. } => item == Some(symbol.as_str()),
                        _ => false,
                    };
                    if taken.projection.is_empty() && place.projection.is_empty() && calls_it {
                        return Some(HandOver { dest: None, part: None, keep: Keep::Called });
                    }
                    let part = (!taken.projection.is_empty()).then(|| describe_place(taken, body, types));
                    return Some(match named(place) {
                        Some(dest) if place.projection.is_empty() => HandOver { dest: Some(dest.clone()), part, keep: Keep::Store { dest } },
                        _ => HandOver { dest: None, part, keep: Keep::Other },
                    });
                }
            }
        }
        if block.terminator_span == at
            && let Terminator::Call { func, args, .. } = &block.terminator
        {
            // `[CLO-6]` — a `once fn` value called: the call uses it up.
            if let FuncRef::Indirect { operand: Operand::Move(place) | Operand::Copy(place), .. } = func
                && place.local == local
                && place.projection.is_empty()
            {
                return Some(HandOver { dest: None, part: None, keep: Keep::Called });
            }
            let mut moved = Vec::new();
            for arg in args {
                moved_by_operand(arg, &mut moved);
            }
            if let Some(taken) = moved.iter().find(|moved| moved.local == local) {
                let part = (!taken.projection.is_empty()).then(|| describe_place(taken, body, types));
                let callee = match func {
                    FuncRef::Direct { symbol, .. } => Some(short(symbol)),
                    _ => None,
                };
                let receiver = args.first().and_then(|arg| match arg {
                    Operand::Move(place) | Operand::Copy(place) if place.local != local => named(place),
                    _ => None,
                });
                return Some(match (receiver, callee) {
                    (Some(receiver), Some(callee)) => HandOver { dest: Some(receiver.clone()), part, keep: Keep::Call { receiver, callee } },
                    (None, Some(callee)) => HandOver { dest: Some(callee), part, keep: Keep::Other },
                    _ => HandOver { dest: None, part, keep: Keep::Other },
                });
            }
        }
    }
    None
}

/// A list, a map or a set (text included): what Python users expect assignment to alias.
fn is_collection(types: &TypeTable, ty: Ty) -> bool {
    let ty = match types.kind(ty) {
        TyKind::Ref { inner, .. } => *inner,
        _ => ty,
    };
    match types.kind(ty) {
        TyKind::Vec { .. } => true,
        TyKind::Struct(id) => types.struct_def(*id).origin.as_ref().is_some_and(|(origin, _)| {
            matches!(origin.as_str(), "std.collections.Map" | "std.collections.Set")
        }),
        _ => false,
    }
}

/// Run over every body.
pub fn elaborate_all(bodies: &mut [Body], types: &TypeTable, sink: &mut Sink) -> usize {
    // `[BLD-7]` — each body on the driver's threads, reported in the bodies' order.
    crate::parallel::each_mut(bodies, sink, |body, sink| elaborate(body, types, sink)).into_iter().map(|(count, _)| count).sum()
}

/// `[THR-5]` — a `Scope` binding may not be moved (`E3014`): its threads borrow the block's places,
/// so it must end where the block does. Every other use of it is a borrow (`scope.spawn`).
pub fn check_scope_moves_all(bodies: &[Body], types: &TypeTable, sink: &mut Sink) -> usize {
    let is_scope = |ty| {
        matches!(types.kind(ty), TyKind::Struct(id)
            if types.struct_def(*id).origin.as_ref().is_some_and(|(origin, _)| matches!(origin.as_str(), "std.thread.Scope" | "std.jobs.JobScope")))
    };
    let mut errors = 0;
    for body in bodies {
        let mut moved = |operand: &Operand, span: Span, sink: &mut Sink| {
            if let Operand::Move(place) = operand
                && place.projection.is_empty()
                && is_scope(body.local(place.local).ty)
            {
                let jobs = matches!(types.kind(body.local(place.local).ty), TyKind::Struct(id)
                    if types.struct_def(*id).origin.as_ref().is_some_and(|(origin, _)| origin.as_str() == "std.jobs.JobScope"));
                let diagnostic = if jobs {
                    Diagnostic::error(codes::E3014, span, "a `JobScope` may not be moved")
                        .help("use it where the `with` binds it; `s.submit` borrows it")
                        .note("a scope's jobs borrow the block's places, so it ends where the block does [JOB-2]")
                } else {
                    Diagnostic::error(codes::E3014, span, "a `Scope` may not be moved")
                        .help("use it where the `with` binds it; `scope.spawn` borrows it")
                        .note("a scope's threads borrow the block's places, so it ends where the block does [THR-5]")
                };
                sink.emit(diagnostic);
                errors += 1;
            }
        };
        for block in &body.blocks {
            for stmt in &block.stmts {
                if let StmtKind::Assign { rvalue, .. } = &stmt.kind {
                    match rvalue {
                        Rvalue::Use(operand) | Rvalue::Cast { operand, .. } => moved(operand, stmt.span, sink),
                        Rvalue::Aggregate { operands, .. } => {
                            for operand in operands {
                                moved(operand, stmt.span, sink);
                            }
                        }
                        _ => {}
                    }
                }
            }
            if let Terminator::Call { args, .. } = &block.terminator {
                for operand in args {
                    moved(operand, block.terminator_span, sink);
                }
            }
        }
    }
    errors
}

#[cfg(test)]
mod tests {
    use super::{Owned, States};

    /// ODR-281 — two paths a byte read, written and joined as a byte each would be (brute force).
    #[test]
    fn packed_states_are_the_states_they_hold() {
        let mut seed = 0x9E37_79B9_7F4A_7C15u64;
        let mut next = move |bound: u64| {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            seed % bound
        };
        for _ in 0..2000 {
            let paths = 1 + next(15) as usize;
            let mut plain = [Vec::new(), Vec::new()];
            let mut packed = [States(vec![0; paths.div_ceil(2)]), States(vec![0; paths.div_ceil(2)])];
            for side in 0..2 {
                for node in 0..paths {
                    let value = Owned(next(8) as u8);
                    plain[side].push(value);
                    packed[side].set(node, Owned(next(8) as u8));
                    packed[side].set(node, value);
                }
                assert!((0..paths).all(|node| packed[side].get(node) == plain[side][node]));
            }
            let joined = packed[0].join(&packed[1]);
            assert!((0..paths).all(|node| joined.get(node) == plain[0][node].join(plain[1][node])));
        }
    }
}
