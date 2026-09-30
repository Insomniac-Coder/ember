//! A loop nest over separate lists, in a function of its own, for MSVC.
//!
//! MSVC turns a nest's two loops around (each element read and written once,
//! its rounds in a register) only when it knows the lists the nest reads and
//! writes are separate, and it believes that only of the `restrict`
//! parameters of a function it keeps separate (measured, ADR-079: adding two
//! lists plus the round into a third, 0.043 s against the hand-written C's
//! 0.014 s, and 0.014 s this way). Ember's lists are separate: two list
//! locals never share their elements, except two borrowed parameters that
//! the nest can only read, where `restrict` promises nothing. So a counted
//! loop whose turn holds a counted loop, that calls nothing and leaves only
//! through its header, whose lists are locals it reaches only by index, and
//! that writes only lists it never reads (where a round reads back what the
//! last one wrote, the reordering makes each element's rounds a chain and
//! is slower), or reads back only a list every round adds the same whole
//! number to (MSVC then adds several rounds at once, ADR-080), is moved into
//! a body of its own (`restrict_views`): each list
//! a view parameter, each length it reads and each value it reads before
//! writing a parameter, and a value that is a constant when the nest starts
//! written in as that constant (a count or a start the program fixes stays
//! fixed, which MSVC needs). The caller makes the views and calls it.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use ember_mir::{
    AssertKind, BasicBlock, BasicBlockId, BinOp, Body, Builtin, Const, FuncRef, LocalDecl, LocalId, LocalKind, Operand,
    ParameterMode, Place, Projection, Rvalue, Stmt, StmtKind, Terminator,
};
use ember_types::{CommonTypes, Ty, TyKind, TypeTable};

use crate::loop_version::{CountedLoop, counted_loop};

/// Moves the qualifying loop nests of every body into bodies of their own,
/// added to `bodies`. Returns how many moved.
pub fn outline_list_kernels_all(bodies: &mut Vec<Body>, types: &mut TypeTable, common: &CommonTypes) -> usize {
    let mut kernels = Vec::new();
    for body in bodies.iter_mut() {
        if body.restrict_views || body.is_abstract || body.is_extern_declaration {
            continue;
        }
        let mut made = 0;
        // One nest at a time: a moved nest's blocks become unreachable, and
        // the counted loops are found again.
        while let Some(kernel) = outline_one(body, types, common, made) {
            kernels.push(kernel);
            made += 1;
        }
    }
    let count = kernels.len();
    bodies.extend(kernels);
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

fn scalar(types: &TypeTable, ty: Ty) -> bool {
    matches!(
        types.kind(ty),
        TyKind::Int(_) | TyKind::Uint(_) | TyKind::Bool | TyKind::Char | TyKind::Float(_) | TyKind::Range(_)
    )
}

/// How a local of the nest is used.
#[derive(Default)]
struct Uses {
    /// Named as a whole value (read or written).
    whole: bool,
    /// Its elements indexed (`xs[i]`), read or written.
    indexed: bool,
    /// An element written.
    element_written: bool,
    /// An element read (or a reference to one taken).
    element_read: bool,
    /// Its length read (`.1` of a list).
    length: bool,
    /// Named any other way (another projection, a reference to it).
    other: bool,
}

fn outline_one(body: &mut Body, types: &mut TypeTable, common: &CommonTypes, made: usize) -> Option<Body> {
    let loops: Vec<CountedLoop> = (0..body.blocks.len()).filter_map(|header| counted_loop(body, types, header)).collect();
    // A nest: a counted loop whose turn holds another counted loop's header.
    let nests: Vec<&CountedLoop> = loops
        .iter()
        .filter(|shape| loops.iter().any(|inner| inner.header != shape.header && shape.region.contains(&inner.header)))
        .collect();
    let outermost: Vec<&CountedLoop> = nests
        .iter()
        .copied()
        .filter(|shape| !nests.iter().any(|outer| outer.header != shape.header && outer.region.contains(&shape.header)))
        .collect();
    for shape in outermost {
        if let Some(kernel) = try_outline(body, types, common, shape, made) {
            return Some(kernel);
        }
    }
    None
}

fn try_outline(
    body: &mut Body,
    types: &mut TypeTable,
    common: &CommonTypes,
    shape: &CountedLoop,
    made: usize,
) -> Option<Body> {
    let mut inside: BTreeSet<usize> = shape.region.iter().copied().collect();
    inside.insert(shape.header);
    // It leaves only through its header, and calls nothing.
    let Terminator::SwitchInt { targets, .. } = &body.blocks[shape.header].terminator else { return None };
    let exit = targets.first()?.1.0 as usize;
    for &block in &inside {
        let terminator = &body.blocks[block].terminator;
        match terminator {
            Terminator::Goto(_) | Terminator::SwitchInt { .. } | Terminator::Assert { .. } | Terminator::Unreachable => {}
            Terminator::Call { .. } | Terminator::Return => return None,
        }
        for next in successors(terminator) {
            if !inside.contains(&next) && !(block == shape.header && next == exit) {
                return None;
            }
        }
        for stmt in &body.blocks[block].stmts {
            match &stmt.kind {
                StmtKind::Assign { .. }
                | StmtKind::CheckedBinaryOp { .. }
                | StmtKind::StorageLive(_)
                | StmtKind::StorageDead(_)
                | StmtKind::Nop => {}
                _ => return None,
            }
        }
    }

    // How the nest names each local.
    let mut uses: BTreeMap<u32, Uses> = BTreeMap::new();
    for &block in &inside {
        let data = &body.blocks[block];
        for stmt in &data.stmts {
            match &stmt.kind {
                StmtKind::Assign { place, rvalue } => {
                    note_place(&mut uses, place, true);
                    for_each_rvalue_place(rvalue, &mut |place, is_ref| {
                        if is_ref {
                            // A reference to an element is an indexing; to
                            // anything else, a use this pass does not follow.
                            if matches!(place.projection.first(), Some(Projection::Index(_))) {
                                note_place(&mut uses, place, false);
                            } else {
                                uses.entry(place.local.0).or_default().other = true;
                            }
                        } else {
                            note_place(&mut uses, place, false);
                        }
                    });
                }
                StmtKind::CheckedBinaryOp { dest, overflow, lhs, rhs, .. } => {
                    note_place(&mut uses, dest, true);
                    note_place(&mut uses, overflow, true);
                    for operand in [lhs, rhs] {
                        if let Operand::Copy(place) | Operand::Move(place) = operand {
                            note_place(&mut uses, place, false);
                        }
                    }
                }
                StmtKind::StorageLive(local) | StmtKind::StorageDead(local) => {
                    uses.entry(local.0).or_default();
                }
                _ => {}
            }
        }
        for_each_terminator_operand(&data.terminator, &mut |operand| {
            if let Operand::Copy(place) | Operand::Move(place) = operand {
                note_place(&mut uses, place, false);
            }
        });
    }

    // The lists: locals of a list type reached only by index or length.
    let mut lists: Vec<LocalId> = Vec::new();
    for (&local, use_) in &uses {
        let ty = body.locals[local as usize].ty;
        if matches!(types.kind(ty), TyKind::Vec { .. }) {
            if use_.whole || use_.other {
                return None;
            }
            if use_.indexed || use_.length {
                lists.push(LocalId(local));
            }
        } else if use_.indexed || use_.length || use_.other {
            // Something else indexed (a view, a fixed array), or reached
            // through a reference: not followed.
            if !matches!(types.kind(ty), TyKind::Ref { .. }) || use_.indexed || use_.length {
                return None;
            }
        }
    }
    let written: Vec<LocalId> = lists.iter().copied().filter(|l| uses[&l.0].element_written).collect();
    if lists.len() < 2 || written.is_empty() {
        return None;
    }
    // A list the nest writes it never reads: then no round builds on an
    // element's last value, and MSVC's reordering (each element taken once,
    // its rounds in a register) only removes memory traffic. Where a round
    // reads back what the last one wrote, the reordering makes each
    // element's rounds a chain no vector instruction can share: measured 2x
    // slower (changing every number using a second list; decimal lists).
    // Except where every round adds the same whole number to each element:
    // then MSVC adds several rounds at once (ADR-080).
    if written
        .iter()
        .any(|&l| uses[&l.0].element_read && !adds_the_same_each_round(body, types, &inside, shape, l, &written))
    {
        return None;
    }
    // Distinct lists: follow the compiler's header copies (`_45 = _4`) back
    // to the list they copy; a list the nest writes shares its elements with
    // no other.
    let root = |mut local: LocalId| -> LocalId {
        for _ in 0..8 {
            let decl = &body.locals[local.0 as usize];
            if decl.kind != LocalKind::Temp {
                break;
            }
            let mut source = None;
            let mut defs = 0;
            for data in &body.blocks {
                for stmt in &data.stmts {
                    if let StmtKind::Assign { place, rvalue } = &stmt.kind
                        && place.local == local
                    {
                        defs += 1;
                        if let Rvalue::Use(Operand::Copy(from) | Operand::Move(from)) = rvalue
                            && from.projection.is_empty()
                        {
                            source = Some(from.local);
                        }
                    }
                }
            }
            match source {
                Some(from) if defs == 1 => local = from,
                _ => break,
            }
        }
        local
    };
    let roots: Vec<LocalId> = lists.iter().map(|&l| root(l)).collect();
    for (i, &list) in lists.iter().enumerate() {
        if !written.contains(&list) {
            continue;
        }
        if roots.iter().enumerate().any(|(j, other)| j != i && *other == roots[i]) {
            return None;
        }
    }

    // Every other local the nest names: a scalar (passed in when its value
    // may come from before the nest), or a local the nest sets before
    // reading it and that nothing reads after it.
    let others: Vec<LocalId> = uses.keys().map(|&l| LocalId(l)).filter(|l| !lists.contains(l)).collect();
    let live_in = live_on_entry(body, &inside, shape.header);
    let mut passed: Vec<LocalId> = Vec::new();
    let mut constants: Vec<(LocalId, Operand)> = Vec::new();
    for &local in &others {
        let decl = &body.locals[local.0 as usize];
        if decl.kind == LocalKind::Return || types.needs_drop(decl.ty) {
            return None;
        }
        let written_inside = writes_in(body, &inside, local);
        if written_inside && live_after(body, exit, local) {
            return None;
        }
        if live_in.contains(&local) {
            if !scalar(types, decl.ty) {
                return None;
            }
            let constant = if written_inside {
                entry_constant(body, &inside, shape.header, local)
            } else {
                single_constant(body, local)
            };
            match constant {
                Some(value) => constants.push((local, value)),
                None => passed.push(local),
            }
        }
    }

    // The new body: views, then lengths, then values; then its own locals.
    let span = body.blocks[shape.header].terminator_span;
    let lengths: Vec<LocalId> = lists.iter().copied().filter(|l| uses[&l.0].length).collect();
    let mut kernel_locals: Vec<LocalDecl> =
        vec![LocalDecl { ty: common.void, kind: LocalKind::Return, name: None, span }];
    let mut map: HashMap<LocalId, LocalId> = HashMap::new();
    let mut view_of: HashMap<LocalId, LocalId> = HashMap::new();
    let mut length_of: HashMap<LocalId, LocalId> = HashMap::new();
    let push = |locals: &mut Vec<LocalDecl>, ty: Ty, kind: LocalKind, name: Option<String>| {
        locals.push(LocalDecl { ty, kind, name, span });
        LocalId(locals.len() as u32 - 1)
    };
    for &list in &lists {
        let view_ty = types.intern(TyKind::Span { elem: elem_of(types, body.locals[list.0 as usize].ty), mutable: written.contains(&list) });
        let name = body.locals[list.0 as usize].name.clone();
        view_of.insert(list, push(&mut kernel_locals, view_ty, LocalKind::Arg, name));
    }
    for &list in &lengths {
        length_of.insert(list, push(&mut kernel_locals, common.usize, LocalKind::Arg, None));
    }
    for &local in &passed {
        let decl = &body.locals[local.0 as usize];
        map.insert(local, push(&mut kernel_locals, decl.ty, LocalKind::Arg, decl.name.clone()));
    }
    let arg_count = kernel_locals.len() - 1;
    for &local in &others {
        if !map.contains_key(&local) {
            let decl = &body.locals[local.0 as usize];
            let kind = if decl.kind == LocalKind::Arg { LocalKind::Temp } else { decl.kind };
            map.insert(local, push(&mut kernel_locals, decl.ty, kind, decl.name.clone()));
        }
    }

    // Its blocks: an entry setting the constants, the nest, and a return
    // where the nest left.
    let order: Vec<usize> = inside.iter().copied().collect();
    let block_of: HashMap<usize, usize> = order.iter().enumerate().map(|(i, &b)| (b, i + 1)).collect();
    let return_block = order.len() + 1;
    let remap_block = |target: BasicBlockId| -> BasicBlockId {
        let old = target.0 as usize;
        BasicBlockId(block_of.get(&old).copied().unwrap_or(return_block) as u32)
    };
    let rewrite = Rewrite { map: &map, view_of: &view_of, length_of: &length_of };
    let mut blocks: Vec<BasicBlock> = Vec::new();
    let entry_stmts: Vec<Stmt> = constants
        .iter()
        .map(|(local, value)| {
            Stmt::new(StmtKind::Assign { place: Place::local(map[local]), rvalue: Rvalue::Use(value.clone()) }, span)
        })
        .collect();
    blocks.push(BasicBlock { stmts: entry_stmts, terminator: Terminator::Goto(remap_block(BasicBlockId(shape.header as u32))), terminator_span: span });
    for &old in &order {
        let data = &body.blocks[old];
        let stmts: Vec<Stmt> = data
            .stmts
            .iter()
            .filter_map(|stmt| rewrite.stmt(stmt))
            .collect();
        let mut terminator = rewrite.terminator(&data.terminator)?;
        crate::loop_version::retarget(&mut terminator, remap_block);
        blocks.push(BasicBlock { stmts, terminator, terminator_span: data.terminator_span });
    }
    blocks.push(BasicBlock { stmts: Vec::new(), terminator: Terminator::Return, terminator_span: span });

    let symbol = format!("{}_loop{made}", body.symbol);
    let mut kernel = body.clone();
    kernel.name = format!("{} (loop {made})", body.name);
    kernel.symbol = symbol.clone();
    kernel.abi = None;
    kernel.locals = kernel_locals;
    kernel.blocks = blocks;
    kernel.arg_count = arg_count;
    kernel.param_modes = vec![ParameterMode::Owned; arg_count];
    kernel.borrows = None;
    kernel.sources = Vec::new();
    kernel.is_lambda = false;
    kernel.emit_if_used = false;
    kernel.borrowed_params = Vec::new();
    kernel.call_argument_bindings = Vec::new();
    kernel.for_iterators = Vec::new();
    kernel.callable_regions = None;
    kernel.closure_environment = None;
    kernel.closure_captures_by_move = false;
    kernel.class_owner = None;
    kernel.class_virtual_slot = None;
    kernel.ffi_counted = None;
    kernel.mut_self = false;
    kernel.elided_accesses = Vec::new();
    kernel.hoisted_accesses = Vec::new();
    kernel.uncounted_handles = Vec::new();
    kernel.removed_checks = Vec::new();
    kernel.restrict_views = true;

    // The caller: the views made, then the call, then on from where the nest
    // left; the nest's own blocks are no longer reached.
    let first = body.blocks.len();
    let mut chain: Vec<BasicBlock> = Vec::new();
    let mut args: Vec<Operand> = Vec::new();
    let new_local = |body: &mut Body, ty: Ty| {
        body.locals.push(LocalDecl { ty, kind: LocalKind::Temp, name: None, span });
        LocalId(body.locals.len() as u32 - 1)
    };
    for &list in &lists {
        let mutable = written.contains(&list);
        let list_ty = body.locals[list.0 as usize].ty;
        let ref_ty = types.intern(TyKind::Ref { mutable, inner: list_ty });
        let view_ty = types.intern(TyKind::Span { elem: elem_of(types, list_ty), mutable });
        let reference = new_local(body, ref_ty);
        let view = new_local(body, view_ty);
        let next = BasicBlockId((first + chain.len() + 1) as u32);
        chain.push(BasicBlock {
            stmts: vec![Stmt::new(
                StmtKind::Assign { place: Place::local(reference), rvalue: Rvalue::Ref { place: Place::local(list), mutable } },
                span,
            )],
            terminator: Terminator::Call {
                func: FuncRef::Builtin { which: Builtin::SpanFrom { mutable }, arg_ty: ref_ty },
                args: vec![Operand::Copy(Place::local(reference))],
                dest: Place::local(view),
                next,
            },
            terminator_span: span,
        });
        args.push(if mutable { Operand::Move(Place::local(view)) } else { Operand::Copy(Place::local(view)) });
    }
    let mut last_stmts = Vec::new();
    for &list in &lengths {
        let length = new_local(body, common.usize);
        last_stmts.push(Stmt::new(
            StmtKind::Assign {
                place: Place::local(length),
                rvalue: Rvalue::Use(Operand::Copy(Place { local: list, projection: vec![Projection::Field(1)] })),
            },
            span,
        ));
        args.push(Operand::Copy(Place::local(length)));
    }
    args.extend(passed.iter().map(|&local| Operand::Copy(Place::local(local))));
    let dest = new_local(body, common.void);
    chain.push(BasicBlock {
        stmts: last_stmts,
        terminator: Terminator::Call {
            func: FuncRef::Direct { symbol, latebound: false },
            args,
            dest: Place::local(dest),
            next: BasicBlockId(exit as u32),
        },
        terminator_span: span,
    });
    let header = BasicBlockId(shape.header as u32);
    let preheader = BasicBlockId(first as u32);
    for block in 0..first {
        if !inside.contains(&block) {
            crate::loop_version::retarget(&mut body.blocks[block].terminator, |target| if target == header { preheader } else { target });
        }
    }
    for &block in &inside {
        body.blocks[block].stmts.clear();
        body.blocks[block].terminator = Terminator::Unreachable;
    }
    body.blocks.extend(chain);
    Some(kernel)
}

/// Whether every round of the nest adds the same whole number to each
/// element of `list`, a list it both reads and writes (ADR-080): its one
/// element read and its one element write are at the inner loop's counter;
/// the value written is that element plus a number no round changes, with no
/// overflow check left on the addition, and at most kept below a power of
/// two (`& 1023`); and a turn takes no branch but the loops' own tests.
/// MSVC then adds several rounds at once (adding `a[i]` five times is adding
/// `5 * a[i]` once). Where a round adds something different (`^ round`), or
/// decimals, whose rounding makes five additions differ from one, it cannot,
/// and the reordering is 2x slower.
fn adds_the_same_each_round(
    body: &Body,
    types: &TypeTable,
    inside: &BTreeSet<usize>,
    outer: &CountedLoop,
    list: LocalId,
    written: &[LocalId],
) -> bool {
    if !matches!(types.kind(elem_of(types, body.locals[list.0 as usize].ty)), TyKind::Int(_) | TyKind::Uint(_)) {
        return false;
    }
    let inner: Vec<CountedLoop> = outer.region.iter().filter_map(|&block| counted_loop(body, types, block)).collect();
    let [inner] = inner.as_slice() else { return false };
    if inside.iter().any(|&block| {
        matches!(body.blocks[block].terminator, Terminator::SwitchInt { .. }) && block != outer.header && block != inner.header
    }) {
        return false;
    }
    let mut inner_inside: BTreeSet<usize> = inner.region.iter().copied().collect();
    inner_inside.insert(inner.header);
    // A value one turn (or round) leaves for the next is not the same in
    // every round; one set earlier in the same turn (or round) is.
    let carried_in_turn = live_on_entry(body, &inner_inside, inner.header);
    let carried_in_round = live_on_entry(body, inside, outer.header);
    // The one statement inside the nest giving `local` its whole value,
    // made before it is read in the same turn or round.
    let definition = |local: LocalId| -> Option<&Rvalue> {
        let mut found = None;
        for &block in inside {
            for stmt in &body.blocks[block].stmts {
                match &stmt.kind {
                    StmtKind::Assign { place, rvalue } if place.local == local => {
                        if found.is_some() || !place.projection.is_empty() {
                            return None;
                        }
                        found = Some((block, rvalue));
                    }
                    StmtKind::CheckedBinaryOp { dest, overflow, .. } if dest.local == local || overflow.local == local => {
                        return None;
                    }
                    _ => {}
                }
            }
        }
        let (block, rvalue) = found?;
        let carried = if inner_inside.contains(&block) { &carried_in_turn } else { &carried_in_round };
        (!carried.contains(&local)).then_some(rvalue)
    };
    // The index is the inner counter itself, or a copy or cast of it.
    fn at_counter<'a>(local: LocalId, counter: LocalId, definition: &dyn Fn(LocalId) -> Option<&'a Rvalue>, depth: usize) -> bool {
        if local == counter {
            return true;
        }
        if depth == 0 {
            return false;
        }
        match definition(local) {
            Some(Rvalue::Use(Operand::Copy(from) | Operand::Move(from)))
            | Some(Rvalue::Cast { operand: Operand::Copy(from) | Operand::Move(from), .. })
                if from.projection.is_empty() =>
            {
                at_counter(from.local, counter, definition, depth - 1)
            }
            _ => false,
        }
    }
    struct Same<'a, 'b> {
        body: &'a Body,
        inside: &'a BTreeSet<usize>,
        inner: LocalId,
        outer: LocalId,
        list: LocalId,
        written: &'a [LocalId],
        definition: &'b dyn Fn(LocalId) -> Option<&'a Rvalue>,
    }
    impl Same<'_, '_> {
        fn local(&self, local: LocalId, depth: usize) -> bool {
            if local == self.inner {
                return true;
            }
            if local == self.outer || depth == 0 {
                return false;
            }
            if !writes_in(self.body, self.inside, local) {
                return true;
            }
            (self.definition)(local).is_some_and(|rvalue| self.rvalue(rvalue, depth - 1))
        }
        fn operand(&self, operand: &Operand, depth: usize) -> bool {
            match operand {
                Operand::Const(_) => true,
                Operand::Copy(place) | Operand::Move(place) if place.projection.is_empty() => self.local(place.local, depth),
                Operand::Copy(place) | Operand::Move(place) => {
                    // A part of a value the nest never changes: an element of
                    // a list it only reads, at an index no round changes.
                    place.local != self.list
                        && !self.written.contains(&place.local)
                        && !writes_in(self.body, self.inside, place.local)
                        && place.projection.iter().all(|step| match step {
                            Projection::Index(index) => self.local(*index, depth),
                            Projection::Field(_) => true,
                            _ => false,
                        })
                }
            }
        }
        fn rvalue(&self, rvalue: &Rvalue, depth: usize) -> bool {
            match rvalue {
                Rvalue::Use(value) | Rvalue::Cast { operand: value, .. } | Rvalue::UnaryOp { operand: value, .. } => {
                    self.operand(value, depth)
                }
                Rvalue::BinaryOp { lhs, rhs, .. } => self.operand(lhs, depth) && self.operand(rhs, depth),
                _ => false,
            }
        }
    }
    let same = Same { body, inside, inner: inner.counter, outer: outer.counter, list, written, definition: &definition };

    // The list's one element read and one element write.
    let mut reads = Vec::new();
    let mut writes = Vec::new();
    for &block in inside {
        let data = &body.blocks[block];
        for stmt in &data.stmts {
            match &stmt.kind {
                StmtKind::Assign { place, rvalue } => {
                    if place.local == list {
                        let [Projection::Index(index)] = place.projection.as_slice() else { return false };
                        writes.push((*index, rvalue));
                    }
                    let mut other = false;
                    for_each_rvalue_place(rvalue, &mut |place, is_ref| {
                        if place.local == list {
                            match place.projection.as_slice() {
                                [Projection::Index(index)] if !is_ref => reads.push(*index),
                                [Projection::Field(1)] if !is_ref => {}
                                _ => other = true,
                            }
                        }
                    });
                    if other {
                        return false;
                    }
                }
                StmtKind::CheckedBinaryOp { dest, overflow, lhs, rhs, .. } => {
                    let names = |operand: &Operand| {
                        matches!(operand, Operand::Copy(place) | Operand::Move(place)
                            if place.local == list && !place.projection.is_empty() && place.projection != [Projection::Field(1)])
                    };
                    if dest.local == list || overflow.local == list || names(lhs) || names(rhs) {
                        return false;
                    }
                }
                _ => {}
            }
        }
        let mut element = false;
        for_each_terminator_operand(&data.terminator, &mut |operand| {
            if let Operand::Copy(place) | Operand::Move(place) = operand
                && place.local == list
                && place.projection != [Projection::Field(1)]
            {
                element = true;
            }
        });
        if element {
            return false;
        }
    }
    let ([read], [(write, value)]) = (reads.as_slice(), writes.as_slice()) else { return false };
    if !at_counter(*read, inner.counter, &definition, 8) || !at_counter(*write, inner.counter, &definition, 8) {
        return false;
    }
    // The value written: the element plus a number no round changes, the
    // sum at most kept below a power of two.
    let mut value: &Rvalue = value;
    for _ in 0..4 {
        let Rvalue::Use(Operand::Copy(from) | Operand::Move(from)) = value else { break };
        match definition(from.local) {
            Some(next) if from.projection.is_empty() => value = next,
            _ => break,
        }
    }
    if let Rvalue::BinaryOp { op: BinOp::BitAnd, lhs, rhs } = value {
        let (sum, mask) = match (lhs, rhs) {
            (Operand::Copy(sum) | Operand::Move(sum), Operand::Const(Const::Int { value: bits, .. }))
            | (Operand::Const(Const::Int { value: bits, .. }), Operand::Copy(sum) | Operand::Move(sum)) => (sum, *bits),
            _ => return false,
        };
        if !sum.projection.is_empty() || !mask.checked_add(1).is_some_and(u128::is_power_of_two) {
            return false;
        }
        let Some(rvalue) = definition(sum.local) else { return false };
        value = rvalue;
    }
    let Rvalue::BinaryOp { op: BinOp::Add, lhs, rhs } = value else { return false };
    let element = |operand: &Operand| {
        matches!(operand, Operand::Copy(place) | Operand::Move(place)
            if place.local == list && place.projection == [Projection::Index(*read)])
    };
    match (element(lhs), element(rhs)) {
        (true, false) => same.operand(rhs, 16),
        (false, true) => same.operand(lhs, 16),
        _ => false,
    }
}

fn elem_of(types: &TypeTable, ty: Ty) -> Ty {
    match types.kind(ty) {
        TyKind::Vec { elem, .. } => *elem,
        _ => unreachable!("a list"),
    }
}

fn note_place(uses: &mut BTreeMap<u32, Uses>, place: &Place, write: bool) {
    let use_ = uses.entry(place.local.0).or_default();
    match place.projection.first() {
        None => use_.whole = true,
        Some(Projection::Index(_)) => {
            use_.indexed = true;
            use_.element_written |= write;
            use_.element_read |= !write;
        }
        Some(Projection::Field(1)) if place.projection.len() == 1 && !write => use_.length = true,
        Some(Projection::Deref) => {
            // Through a reference: the reference is read as a whole.
            use_.whole = true;
        }
        _ => use_.other = true,
    }
    for step in &place.projection {
        if let Projection::Index(index) = step {
            uses.entry(index.0).or_default().whole = true;
        }
    }
}

fn for_each_rvalue_place(rvalue: &Rvalue, f: &mut dyn FnMut(&Place, bool)) {
    let operand = |value: &Operand, f: &mut dyn FnMut(&Place, bool)| {
        if let Operand::Copy(place) | Operand::Move(place) = value {
            f(place, false);
        }
    };
    match rvalue {
        Rvalue::Use(value) | Rvalue::Cast { operand: value, .. } | Rvalue::UnaryOp { operand: value, .. } => operand(value, f),
        Rvalue::BinaryOp { lhs, rhs, .. } => {
            operand(lhs, f);
            operand(rhs, f);
        }
        Rvalue::Aggregate { operands, .. } => operands.iter().for_each(|value| operand(value, f)),
        Rvalue::Repeat { value, .. } => operand(value, f),
        Rvalue::Ref { place, .. } => f(place, true),
        Rvalue::Discriminant(place) => f(place, false),
    }
}

fn for_each_terminator_operand(terminator: &Terminator, f: &mut dyn FnMut(&Operand)) {
    match terminator {
        Terminator::SwitchInt { discr, .. } => f(discr),
        Terminator::Assert { cond, msg, .. } => {
            f(cond);
            match msg {
                AssertKind::Bounds { len, index } => {
                    f(len);
                    f(index);
                }
                AssertKind::RefCellBorrow { file, line } => {
                    f(file);
                    f(line);
                }
                AssertKind::Panic { message } => f(message),
                _ => {}
            }
        }
        Terminator::Call { args, .. } => args.iter().for_each(|arg| f(arg)),
        Terminator::Goto(_) | Terminator::Return | Terminator::Unreachable => {}
    }
}

/// The value `local` is given at its one assignment in the body, when that
/// is a constant.
fn single_constant(body: &Body, local: LocalId) -> Option<Operand> {
    let mut found = None;
    for data in &body.blocks {
        for stmt in &data.stmts {
            match &stmt.kind {
                StmtKind::Assign { place, rvalue } if place.local == local => {
                    let Rvalue::Use(value @ Operand::Const(Const::Int { .. } | Const::Bool(_))) = rvalue else { return None };
                    if found.is_some() || !place.projection.is_empty() {
                        return None;
                    }
                    found = Some(value.clone());
                }
                StmtKind::CheckedBinaryOp { dest, overflow, .. } if dest.local == local || overflow.local == local => {
                    return None;
                }
                _ => {}
            }
        }
        if let Terminator::Call { dest, .. } = &data.terminator
            && dest.local == local
        {
            return None;
        }
    }
    found
}

/// The constant `local` holds whenever the nest starts: the one path into
/// its header (each block before it reached from one block only) sets it to
/// that constant, and nothing after that sets it again. A loop counter that
/// starts at a number the program wrote is one (`for round in 0..2000`).
fn entry_constant(body: &Body, inside: &BTreeSet<usize>, header: usize, local: LocalId) -> Option<Operand> {
    let mut predecessors = vec![Vec::new(); body.blocks.len()];
    for (block, data) in body.blocks.iter().enumerate() {
        for next in successors(&data.terminator) {
            predecessors[next].push(block);
        }
    }
    let outside: Vec<usize> = predecessors[header].iter().copied().filter(|b| !inside.contains(b)).collect();
    let [first] = outside.as_slice() else { return None };
    let mut block = *first;
    for _ in 0..64 {
        let data = &body.blocks[block];
        if let Terminator::Call { dest, .. } = &data.terminator
            && dest.local == local
        {
            return None;
        }
        for stmt in data.stmts.iter().rev() {
            match &stmt.kind {
                StmtKind::Assign { place, rvalue } if place.local == local => {
                    return match rvalue {
                        Rvalue::Use(value @ Operand::Const(Const::Int { .. } | Const::Bool(_))) if place.projection.is_empty() => {
                            Some(value.clone())
                        }
                        _ => None,
                    };
                }
                StmtKind::CheckedBinaryOp { dest, overflow, .. } if dest.local == local || overflow.local == local => return None,
                _ => {}
            }
        }
        let [from] = predecessors[block].as_slice() else { return None };
        block = *from;
    }
    None
}

fn writes_in(body: &Body, inside: &BTreeSet<usize>, local: LocalId) -> bool {
    inside.iter().any(|&block| {
        body.blocks[block].stmts.iter().any(|stmt| match &stmt.kind {
            StmtKind::Assign { place, .. } => place.local == local,
            StmtKind::CheckedBinaryOp { dest, overflow, .. } => dest.local == local || overflow.local == local,
            _ => false,
        })
    })
}

/// Reads and whole writes of one statement, in the order they happen.
fn stmt_effect(stmt: &Stmt, local: LocalId, locals: usize) -> (bool, bool) {
    let mut read = vec![false; locals];
    crate::strength_reduce::stmt_reads(stmt, &mut read);
    let reads = read[local.0 as usize];
    let writes = match &stmt.kind {
        StmtKind::Assign { place, .. } => place.local == local && place.projection.is_empty(),
        StmtKind::CheckedBinaryOp { dest, overflow, .. } => {
            (dest.local == local && dest.projection.is_empty()) || (overflow.local == local && overflow.projection.is_empty())
        }
        StmtKind::StorageDead(dead) | StmtKind::StorageLive(dead) => *dead == local,
        _ => false,
    };
    (reads, writes)
}

/// Whether some path from the start of `from` reads `local` before giving it
/// a new value.
fn live_after(body: &Body, from: usize, local: LocalId) -> bool {
    let mut seen = HashSet::new();
    let mut work = vec![from];
    while let Some(block) = work.pop() {
        if !seen.insert(block) {
            continue;
        }
        let data = &body.blocks[block];
        let mut killed = false;
        for stmt in &data.stmts {
            let mut read = vec![false; body.locals.len()];
            crate::strength_reduce::stmt_reads(stmt, &mut read);
            if read[local.0 as usize] {
                return true;
            }
            if stmt_effect(stmt, local, body.locals.len()).1 {
                killed = true;
                break;
            }
        }
        if killed {
            continue;
        }
        let mut read = vec![false; body.locals.len()];
        crate::strength_reduce::terminator_reads(&data.terminator, &mut read);
        if read[local.0 as usize] {
            return true;
        }
        if let Terminator::Call { dest, .. } = &data.terminator
            && dest.local == local
        {
            continue;
        }
        work.extend(successors(&data.terminator));
    }
    false
}

/// The locals the nest may read, starting at its header, before it gives
/// them a value.
fn live_on_entry(body: &Body, inside: &BTreeSet<usize>, header: usize) -> HashSet<LocalId> {
    let n = body.locals.len();
    let mut live_in: HashMap<usize, Vec<bool>> = inside.iter().map(|&b| (b, vec![false; n])).collect();
    loop {
        let mut changed = false;
        for &block in inside.iter().rev() {
            let data = &body.blocks[block];
            let mut live = vec![false; n];
            for next in successors(&data.terminator) {
                if let Some(next_live) = live_in.get(&next) {
                    for (slot, &value) in live.iter_mut().zip(next_live) {
                        *slot |= value;
                    }
                }
            }
            crate::strength_reduce::terminator_reads(&data.terminator, &mut live);
            for stmt in data.stmts.iter().rev() {
                // A whole write ends the value's life; then the statement's
                // reads begin one.
                match &stmt.kind {
                    StmtKind::Assign { place, .. } if place.projection.is_empty() => live[place.local.0 as usize] = false,
                    StmtKind::CheckedBinaryOp { dest, overflow, .. } => {
                        if dest.projection.is_empty() {
                            live[dest.local.0 as usize] = false;
                        }
                        if overflow.projection.is_empty() {
                            live[overflow.local.0 as usize] = false;
                        }
                    }
                    _ => {}
                }
                crate::strength_reduce::stmt_reads(stmt, &mut live);
            }
            if live_in[&block] != live {
                live_in.insert(block, live);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    live_in[&header].iter().enumerate().filter(|(_, l)| **l).map(|(i, _)| LocalId(i as u32)).collect()
}

/// A statement or terminator of the nest, in the new body's locals: a list
/// indexed becomes its view indexed, a length read becomes the length
/// parameter.
struct Rewrite<'a> {
    map: &'a HashMap<LocalId, LocalId>,
    view_of: &'a HashMap<LocalId, LocalId>,
    length_of: &'a HashMap<LocalId, LocalId>,
}

impl Rewrite<'_> {
    fn local(&self, local: LocalId) -> LocalId {
        self.map.get(&local).copied().or_else(|| self.view_of.get(&local).copied()).expect("every local of the nest is mapped")
    }

    fn place(&self, place: &Place) -> Place {
        let local = self.local(place.local);
        let projection = place
            .projection
            .iter()
            .map(|step| match step {
                Projection::Index(index) => Projection::Index(self.local(*index)),
                other => other.clone(),
            })
            .collect();
        Place { local, projection }
    }

    fn operand(&self, operand: &Operand) -> Operand {
        match operand {
            Operand::Copy(place) | Operand::Move(place)
                if place.projection == [Projection::Field(1)] && self.length_of.contains_key(&place.local) =>
            {
                Operand::Copy(Place::local(self.length_of[&place.local]))
            }
            Operand::Copy(place) => Operand::Copy(self.place(place)),
            Operand::Move(place) => Operand::Move(self.place(place)),
            Operand::Const(value) => Operand::Const(value.clone()),
        }
    }

    fn rvalue(&self, rvalue: &Rvalue) -> Rvalue {
        match rvalue {
            Rvalue::Use(value) => Rvalue::Use(self.operand(value)),
            Rvalue::Cast { operand, kind, to } => Rvalue::Cast { operand: self.operand(operand), kind: kind.clone(), to: *to },
            Rvalue::UnaryOp { op, operand } => Rvalue::UnaryOp { op: *op, operand: self.operand(operand) },
            Rvalue::BinaryOp { op, lhs, rhs } => Rvalue::BinaryOp { op: *op, lhs: self.operand(lhs), rhs: self.operand(rhs) },
            Rvalue::Aggregate { kind, operands } => {
                Rvalue::Aggregate { kind: kind.clone(), operands: operands.iter().map(|value| self.operand(value)).collect() }
            }
            Rvalue::Repeat { value, count } => Rvalue::Repeat { value: self.operand(value), count: *count },
            Rvalue::Ref { place, mutable } => Rvalue::Ref { place: self.place(place), mutable: *mutable },
            Rvalue::Discriminant(place) => Rvalue::Discriminant(self.place(place)),
        }
    }

    fn stmt(&self, stmt: &Stmt) -> Option<Stmt> {
        let kind = match &stmt.kind {
            StmtKind::Assign { place, rvalue } => StmtKind::Assign { place: self.place(place), rvalue: self.rvalue(rvalue) },
            StmtKind::CheckedBinaryOp { op, dest, overflow, lhs, rhs } => StmtKind::CheckedBinaryOp {
                op: *op,
                dest: self.place(dest),
                overflow: self.place(overflow),
                lhs: self.operand(lhs),
                rhs: self.operand(rhs),
            },
            StmtKind::StorageLive(local) | StmtKind::StorageDead(local) if self.view_of.contains_key(local) => return None,
            StmtKind::StorageLive(local) => StmtKind::StorageLive(self.local(*local)),
            StmtKind::StorageDead(local) => StmtKind::StorageDead(self.local(*local)),
            StmtKind::Nop => StmtKind::Nop,
            _ => unreachable!("the nest holds only these"),
        };
        let mut new = stmt.clone();
        new.kind = kind;
        Some(new)
    }

    fn terminator(&self, terminator: &Terminator) -> Option<Terminator> {
        Some(match terminator {
            Terminator::Goto(next) => Terminator::Goto(*next),
            Terminator::SwitchInt { discr, targets, otherwise } => {
                Terminator::SwitchInt { discr: self.operand(discr), targets: targets.clone(), otherwise: *otherwise }
            }
            Terminator::Assert { cond, expected, msg, next, span } => {
                let msg = match msg {
                    AssertKind::Bounds { len, index } => AssertKind::Bounds { len: self.operand(len), index: self.operand(index) },
                    AssertKind::RefCellBorrow { file, line } => {
                        AssertKind::RefCellBorrow { file: self.operand(file), line: self.operand(line) }
                    }
                    AssertKind::Panic { message } => AssertKind::Panic { message: self.operand(message) },
                    other => other.clone(),
                };
                Terminator::Assert { cond: self.operand(cond), expected: *expected, msg, next: *next, span: *span }
            }
            Terminator::Unreachable => Terminator::Unreachable,
            Terminator::Call { .. } | Terminator::Return => return None,
        })
    }
}
