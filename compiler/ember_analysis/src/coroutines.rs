//! `[CORO-4]`–`[CORO-7]`, `[CORO-9]`, `[CORO-10]`, `[CORO-12]`, `[CORO-13]` — the
//! coroutine transform (ADR-193).
//!
//! The checker writes a `gen fn` as a frame struct, a constructor building it,
//! stubs for the frame's `next`, `drop` and `result`, and the body as a method
//! of the frame (`ember_typeck::coroutines`). Lowering makes each suspension
//! point a call answering whether the generator was resumed, with a dropped
//! path that ends the body the way a `return` does, minus its `defer`s
//! (`ember_mir::lower::coroutines`). Every analysis up to the borrow check sees
//! that body as an ordinary function, so drop elaboration decides exactly what
//! a suspended frame drops.
//!
//! Then `check_all` reports what may not be held across a `yield` (`E2221`,
//! `E2229`, `E2231`) and a frame that would hold itself (`E2200`), and
//! `transform_all` rewrites the body into a state machine over the frame, as a
//! C programmer writes one by hand: a `state` field, `next` switching on it to
//! the code after the suspension it left at, each `yield` storing the state
//! and returning `Some(value)`. A local live across a suspension becomes a
//! frame field and every other stays a local of the function using it
//! (`[CORO-4]`); the frame's size is fixed and nothing allocates (`[CORO-5]`).
//! The frame's `drop` switches on the same state to the dropped path of that
//! suspension point (`[CORO-7]`).

use std::collections::{BTreeSet, HashMap, HashSet};

use ember_diag::{Diagnostic, Sink, codes};
use ember_mir::lower::coroutines::{is_start, is_suspension, is_yield, resumed_symbol};
use ember_mir::{
    AggregateKind, AssertKind, BasicBlock, BasicBlockId, Body, Const, FuncRef, LocalDecl, LocalId, LocalKind,
    Operand, Place, Projection, RETURN_LOCAL, Rvalue, Stmt, StmtKind, Terminator,
};
use ember_span::{Span, Symbol};
use ember_types::{FieldDef, FieldVis, StructId, Ty, TyKind, TypeTable, UintTy};

use crate::regions::project_type;

/// One suspension point: the call, the local it answers into, the block the
/// call returns to and the switch on the answer, with its two targets.
#[derive(Clone, Debug)]
struct Point {
    call: usize,
    resumed: LocalId,
    next: usize,
    switch: usize,
    dropped: usize,
    resume: usize,
    /// The yielded value; `None` for the start.
    value: Option<Operand>,
    span: Span,
}

/// A generator's body as lowering left it.
struct Generator {
    body: usize,
    frame: StructId,
    frame_ty: Ty,
    /// The body's parameters after the frame, in order: the frame's fields
    /// `0..`, as the constructor stored them.
    params: Vec<LocalId>,
    /// The start first, then each `yield` in block order.
    points: Vec<Point>,
}

fn find_generators(bodies: &[Body], types: &TypeTable) -> Vec<Generator> {
    let mut found = Vec::new();
    for (index, body) in bodies.iter().enumerate() {
        let Some(frame_ty) = frame_param(body, types) else { continue };
        let mut start = None;
        let mut yields = Vec::new();
        for (block_index, block) in body.blocks.iter().enumerate() {
            let Terminator::Call { func: FuncRef::Direct { symbol, .. }, args, dest, next } = &block.terminator else { continue };
            if !is_suspension(symbol) {
                continue;
            }
            let Some(resumed) = dest.is_local().then_some(dest.local) else { continue };
            let Some((switch, dropped, resume)) = find_switch(body, next.0 as usize, resumed) else { continue };
            let point = Point {
                call: block_index,
                resumed,
                next: next.0 as usize,
                switch,
                dropped,
                resume,
                value: if is_yield(symbol) { args.first().cloned() } else { None },
                span: block.terminator_span,
            };
            if is_start(symbol) {
                start = Some(point);
            } else {
                yields.push(point);
            }
        }
        let Some(start) = start else { continue };
        let TyKind::Struct(frame) = *types.kind(frame_ty) else { continue };
        let mut points = vec![start];
        points.extend(yields);
        let params = (2..=body.arg_count).map(|index| LocalId(index as u32)).collect();
        found.push(Generator { body: index, frame, frame_ty, params, points });
    }
    found
}

/// A generator body's first parameter: `mut self` of its frame.
fn frame_param(body: &Body, types: &TypeTable) -> Option<Ty> {
    let decl = body.locals.get(1).filter(|_| body.arg_count >= 1)?;
    match types.kind(decl.ty) {
        TyKind::Ref { mutable: true, inner } if types.is_coroutine_frame(*inner) => Some(*inner),
        _ => None,
    }
}

/// From the block a suspension call returns to, the switch on its answer.
fn find_switch(body: &Body, mut block: usize, resumed: LocalId) -> Option<(usize, usize, usize)> {
    for _ in 0..body.blocks.len() {
        match &body.blocks[block].terminator {
            Terminator::SwitchInt { discr: Operand::Copy(place), targets, otherwise }
                if place.is_local() && place.local == resumed && targets.len() == 1 && targets[0].0 == 0 =>
            {
                return Some((block, targets[0].1.0 as usize, otherwise.0 as usize));
            }
            Terminator::Goto(next) => block = next.0 as usize,
            _ => return None,
        }
    }
    None
}

// -- liveness --------------------------------------------------------------

/// A set of locals.
#[derive(Clone, PartialEq, Eq)]
struct Locals(Vec<u64>);

impl Locals {
    fn new(count: usize) -> Locals {
        Locals(vec![0; count.div_ceil(64)])
    }
    fn insert(&mut self, local: LocalId) {
        let i = local.0 as usize;
        self.0[i / 64] |= 1 << (i % 64);
    }
    fn remove(&mut self, local: LocalId) {
        let i = local.0 as usize;
        self.0[i / 64] &= !(1 << (i % 64));
    }
    fn union(&mut self, other: &Locals) {
        for (a, b) in self.0.iter_mut().zip(&other.0) {
            *a |= b;
        }
    }
    fn iter(&self) -> impl Iterator<Item = LocalId> + '_ {
        (0..self.0.len() * 64).filter(|&i| self.0[i / 64] & (1 << (i % 64)) != 0).map(|i| LocalId(i as u32))
    }
}

fn successors(terminator: &Terminator) -> Vec<usize> {
    match terminator {
        Terminator::Goto(target) => vec![target.0 as usize],
        Terminator::SwitchInt { targets, otherwise, .. } => {
            targets.iter().map(|(_, target)| target.0 as usize).chain([otherwise.0 as usize]).collect()
        }
        Terminator::Call { next, .. } | Terminator::Assert { next, .. } => vec![next.0 as usize],
        Terminator::Return | Terminator::Unreachable => Vec::new(),
    }
}

fn use_place(place: &Place, live: &mut Locals) {
    live.insert(place.local);
    for projection in &place.projection {
        if let Projection::Index(index) = projection {
            live.insert(*index);
        }
    }
}

fn use_operand(operand: &Operand, live: &mut Locals) {
    if let Operand::Copy(place) | Operand::Move(place) = operand {
        use_place(place, live);
    }
}

fn use_rvalue(rvalue: &Rvalue, live: &mut Locals) {
    match rvalue {
        Rvalue::Use(operand) | Rvalue::UnaryOp { operand, .. } | Rvalue::Cast { operand, .. } | Rvalue::Repeat { value: operand, .. } => {
            use_operand(operand, live)
        }
        Rvalue::BinaryOp { lhs, rhs, .. } => {
            use_operand(lhs, live);
            use_operand(rhs, live);
        }
        Rvalue::Aggregate { operands, .. } => operands.iter().for_each(|operand| use_operand(operand, live)),
        Rvalue::Discriminant(place) | Rvalue::Ref { place, .. } => use_place(place, live),
    }
}

/// A write of `place`: a whole local is defined; a part of one leaves the
/// rest as it was, so the local is read.
fn define(place: &Place, live: &mut Locals) {
    if place.is_local() {
        live.remove(place.local);
    } else {
        use_place(place, live);
    }
}

fn assert_operands(msg: &AssertKind) -> Vec<&Operand> {
    match msg {
        AssertKind::Bounds { len, index } => vec![len, index],
        AssertKind::RefCellBorrow { file, line } => vec![file, line],
        AssertKind::Panic { message } => vec![message],
        _ => Vec::new(),
    }
}

/// What each block needs live on entry. The return slot is never live across
/// a suspension (a `return` writes it last), so `Return` reads nothing.
fn live_in(body: &Body) -> Vec<Locals> {
    let count = body.locals.len();
    let mut live_in = vec![Locals::new(count); body.blocks.len()];
    let mut changed = true;
    while changed {
        changed = false;
        for index in (0..body.blocks.len()).rev() {
            let block = &body.blocks[index];
            let mut live = Locals::new(count);
            for successor in successors(&block.terminator) {
                live.union(&live_in[successor]);
            }
            match &block.terminator {
                Terminator::Call { func, args, dest, .. } => {
                    define(dest, &mut live);
                    args.iter().for_each(|arg| use_operand(arg, &mut live));
                    if let FuncRef::Indirect { operand, .. } = func {
                        use_operand(operand, &mut live);
                    }
                }
                Terminator::SwitchInt { discr, .. } => use_operand(discr, &mut live),
                Terminator::Assert { cond, msg, .. } => {
                    use_operand(cond, &mut live);
                    assert_operands(msg).into_iter().for_each(|operand| use_operand(operand, &mut live));
                }
                Terminator::Goto(_) | Terminator::Return | Terminator::Unreachable => {}
            }
            for stmt in block.stmts.iter().rev() {
                match &stmt.kind {
                    StmtKind::Assign { place, rvalue } => {
                        define(place, &mut live);
                        use_rvalue(rvalue, &mut live);
                    }
                    StmtKind::CheckedBinaryOp { dest, overflow, lhs, rhs, .. } => {
                        define(dest, &mut live);
                        define(overflow, &mut live);
                        use_operand(lhs, &mut live);
                        use_operand(rhs, &mut live);
                    }
                    StmtKind::Drop { place, flag, .. } => {
                        use_place(place, &mut live);
                        if let Some(flag) = flag {
                            live.insert(*flag);
                        }
                    }
                    StmtKind::BeginAccess { place, .. }
                    | StmtKind::BeginAccessTransfer { place, .. }
                    | StmtKind::EndAccess { place, .. }
                    | StmtKind::EndAccessTransfer { place, .. } => use_place(place, &mut live),
                    StmtKind::StorageLive(_) | StmtKind::StorageDead(_) | StmtKind::Nop => {}
                }
            }
            if live != live_in[index] {
                live_in[index] = live;
                changed = true;
            }
        }
    }
    live_in
}

/// What must outlive suspension `point`: what the code after it reads,
/// resumed or dropped.
fn live_across(live_in: &[Locals], point: &Point) -> Locals {
    let mut live = live_in[point.next].clone();
    live.remove(point.resumed);
    live.remove(RETURN_LOCAL);
    live
}

// -- what a local may point into (`[CORO-6]`) --------------------------------

/// Storage a borrow may point into: a local of the generator's own frame, or
/// a field of a class object (`[CORO-12]`).
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
enum Target {
    Local(LocalId),
    ClassField,
}

/// For each local, the storage its value may point into, through every
/// assignment, call and store (flow-insensitive, as the region analysis is,
/// ADR-010). A call's result points where the callee's region summary says
/// (`[MIR-REG-1]`), or else where its source parameters' do (`[LT-1]`). Only a
/// local whose type can hold a borrow points anywhere.
fn points_to(body: &Body, types: &TypeTable, callees: &HashMap<&str, &Body>) -> Vec<BTreeSet<Target>> {
    let mut pts: Vec<BTreeSet<Target>> = vec![BTreeSet::new(); body.locals.len()];
    let holds: Vec<bool> = body.locals.iter().map(|decl| may_borrow(types, decl.ty)).collect();
    let mut changed = true;
    while changed {
        changed = false;
        let mut add = |pts: &mut Vec<BTreeSet<Target>>, local: LocalId, from: &BTreeSet<Target>| {
            if !holds[local.0 as usize] {
                return;
            }
            let set = &mut pts[local.0 as usize];
            let before = set.len();
            set.extend(from.iter().copied());
            if set.len() != before {
                changed = true;
            }
        };
        for block in &body.blocks {
            for stmt in &block.stmts {
                let StmtKind::Assign { place, rvalue } = &stmt.kind else { continue };
                let from = rvalue_targets(body, types, &pts, rvalue);
                if from.is_empty() {
                    continue;
                }
                for local in store_targets(body, types, &pts, place) {
                    add(&mut pts, local, &from);
                }
            }
            if let Terminator::Call { func, args, dest, .. } = &block.terminator {
                let from = call_targets(body, types, &pts, callees, func, args);
                if !from.is_empty() {
                    for local in store_targets(body, types, &pts, dest) {
                        add(&mut pts, local, &from);
                    }
                }
                // A callee may store what its other arguments are, or hold,
                // into what a `ref mut` or a `MutSpan` argument points to (a
                // borrow of that storage itself it cannot: it does not outlive
                // the call).
                let given: Vec<BTreeSet<Target>> = args
                    .iter()
                    .map(|arg| {
                        let value = operand_targets(body, types, &pts, arg);
                        let mut out = contents(&pts, &value);
                        out.extend(value);
                        out
                    })
                    .collect();
                for (index, arg) in args.iter().enumerate() {
                    let mutable = matches!(arg, Operand::Copy(place) | Operand::Move(place)
                        if matches!(types.kind(place_ty(body, types, place)), TyKind::Ref { mutable: true, .. } | TyKind::Span { mutable: true, .. }));
                    if !mutable {
                        continue;
                    }
                    let others: BTreeSet<Target> =
                        given.iter().enumerate().filter(|&(other, _)| other != index).flat_map(|(_, set)| set.iter().copied()).collect();
                    if others.is_empty() {
                        continue;
                    }
                    for target in operand_targets(body, types, &pts, arg) {
                        if let Target::Local(local) = target {
                            add(&mut pts, local, &others);
                        }
                    }
                }
            }
        }
    }
    pts
}

/// Whether a value of `ty` can hold a borrow.
fn may_borrow(types: &TypeTable, ty: Ty) -> bool {
    types.is_view(ty) || matches!(types.kind(ty), TyKind::Ptr { .. } | TyKind::Fn { stateless: false, .. } | TyKind::Dyn { .. })
}

fn place_ty(body: &Body, types: &TypeTable, place: &Place) -> Ty {
    crate::regions::place_type(body, types, place)
}

/// How a place reaches its storage: in its local's own storage, through a
/// pointer the local holds, or into a class object.
#[derive(PartialEq, Eq)]
enum Reach {
    Own,
    Pointer,
    Object,
}

fn reach(body: &Body, types: &TypeTable, place: &Place) -> Reach {
    let mut ty = body.local(place.local).ty;
    let mut variant = None;
    for projection in &place.projection {
        match (projection, types.kind(ty)) {
            (Projection::Deref, _) => return Reach::Pointer,
            (Projection::Index(_) | Projection::ConstIndex(_), TyKind::Span { .. } | TyKind::Ptr { .. } | TyKind::Str) => {
                return Reach::Pointer;
            }
            (Projection::Field(_), TyKind::Class(_)) => return Reach::Object,
            _ => {}
        }
        ty = project_type(types, ty, projection, &mut variant);
    }
    Reach::Own
}

/// What the storage `targets` holds may point into, one level down.
fn contents(pts: &[BTreeSet<Target>], targets: &BTreeSet<Target>) -> BTreeSet<Target> {
    let mut out = BTreeSet::new();
    for target in targets {
        if let Target::Local(local) = target {
            out.extend(pts[local.0 as usize].iter().copied());
        }
    }
    out
}

/// What a value read from `place` may point into: its local's, or, read
/// through a pointer, what the storage it points to holds. A class object's
/// fields hold no borrow (`[TYP-15]`).
fn place_targets(body: &Body, types: &TypeTable, pts: &[BTreeSet<Target>], place: &Place) -> BTreeSet<Target> {
    let own = &pts[place.local.0 as usize];
    match reach(body, types, place) {
        Reach::Own => own.clone(),
        Reach::Pointer => contents(pts, own),
        Reach::Object => BTreeSet::new(),
    }
}

fn operand_targets(body: &Body, types: &TypeTable, pts: &[BTreeSet<Target>], operand: &Operand) -> BTreeSet<Target> {
    match operand {
        Operand::Copy(place) | Operand::Move(place) => place_targets(body, types, pts, place),
        Operand::Const(_) => BTreeSet::new(),
    }
}

/// What a borrow of `place` points into.
fn borrow_targets(body: &Body, types: &TypeTable, pts: &[BTreeSet<Target>], place: &Place) -> BTreeSet<Target> {
    match reach(body, types, place) {
        Reach::Own => BTreeSet::from([Target::Local(place.local)]),
        Reach::Pointer => pts[place.local.0 as usize].clone(),
        Reach::Object => BTreeSet::from([Target::ClassField]),
    }
}

fn rvalue_targets(body: &Body, types: &TypeTable, pts: &[BTreeSet<Target>], rvalue: &Rvalue) -> BTreeSet<Target> {
    match rvalue {
        Rvalue::Use(operand) | Rvalue::UnaryOp { operand, .. } | Rvalue::Cast { operand, .. } | Rvalue::Repeat { value: operand, .. } => {
            operand_targets(body, types, pts, operand)
        }
        Rvalue::BinaryOp { lhs, rhs, .. } => {
            let mut out = operand_targets(body, types, pts, lhs);
            out.extend(operand_targets(body, types, pts, rhs));
            out
        }
        Rvalue::Aggregate { operands, .. } => operands.iter().flat_map(|operand| operand_targets(body, types, pts, operand)).collect(),
        Rvalue::Ref { place, .. } => borrow_targets(body, types, pts, place),
        Rvalue::Discriminant(_) => BTreeSet::new(),
    }
}

/// What a call's result may point into.
fn call_targets(
    body: &Body,
    types: &TypeTable,
    pts: &[BTreeSet<Target>],
    callees: &HashMap<&str, &Body>,
    func: &FuncRef,
    args: &[Operand],
) -> BTreeSet<Target> {
    // An argument's value, and what the storage it points to holds.
    let both = |index: usize| {
        let mut out = BTreeSet::new();
        if let Some(arg) = args.get(index) {
            let value = operand_targets(body, types, pts, arg);
            out.extend(contents(pts, &value));
            out.extend(value);
        }
        out
    };
    let sources: Option<Vec<usize>> = match func {
        FuncRef::Direct { symbol, .. } => match callees.get(symbol.as_str()) {
            Some(callee) => {
                if let Some(result) = callee.callable_regions.as_ref().and_then(|metadata| metadata.result.as_ref()) {
                    let mut out = BTreeSet::new();
                    for field in &result.fields {
                        for source in &field.sources {
                            if let ember_mir::ResultRegionSource::View { argument, projection } = source
                                && let Some(arg) = args.get(*argument)
                            {
                                let value = operand_targets(body, types, pts, arg);
                                out.extend(if projection.contains(&Projection::Deref) { contents(pts, &value) } else { value });
                            }
                        }
                    }
                    return out;
                }
                Some(callee.sources.clone())
            }
            None => None,
        },
        FuncRef::Indirect { sources, .. } => sources.clone(),
        _ => None,
    };
    match sources {
        Some(sources) => sources.into_iter().flat_map(both).collect(),
        None => (0..args.len()).flat_map(both).collect(),
    }
}

/// The locals whose storage a write to `place` changes.
fn store_targets(body: &Body, types: &TypeTable, pts: &[BTreeSet<Target>], place: &Place) -> Vec<LocalId> {
    match reach(body, types, place) {
        Reach::Own => vec![place.local],
        Reach::Pointer => pts[place.local.0 as usize]
            .iter()
            .filter_map(|target| match target {
                Target::Local(local) => Some(*local),
                Target::ClassField => None,
            })
            .collect(),
        Reach::Object => Vec::new(),
    }
}

// -- checks ----------------------------------------------------------------

/// `[CORO-6]`, `[CORO-12]`, `[CORO-13]`, `[CORO-10]`, `[CORO-5]` — what a
/// generator may not hold across a `yield`, a frame that would hold itself,
/// and a generator given to C. Returns how many errors it reported.
pub fn check_all(bodies: &[Body], types: &TypeTable, sink: &mut Sink) -> usize {
    let generators = find_generators(bodies, types);
    let callees: HashMap<&str, &Body> = bodies.iter().map(|body| (body.symbol.as_str(), body)).collect();
    let mut errors = 0;
    let mut holds: Vec<(StructId, Vec<StructId>, Span)> = Vec::new();
    for generator in &generators {
        let body = &bodies[generator.body];
        let name = |local: LocalId| body.local(local).name.clone().filter(|name| !name.starts_with('_') && !name.contains('$'));
        // `[CORO-12]` — a class generator method takes `self`.
        if let Some(&receiver) = generator.params.first()
            && body.local(receiver).name.as_deref() == Some("__self")
            && matches!(types.kind(body.local(receiver).ty), TyKind::Ref { mutable: true, inner } if matches!(types.kind(*inner), TyKind::Class(_)))
        {
            sink.emit(
                Diagnostic::error(codes::E2229, body.span, "a class's generator method takes `self`, not `mut self`")
                    .note("a suspended generator would keep a write access to the whole object open between its steps; each access inside the body is checked on its own instead [CORO-12]"),
            );
            errors += 1;
        }
        // How the body reaches each parameter in the frame (`lay_out`): held
        // as the body names it, or as the `ref` it reads through.
        for (index, &param) in generator.params.iter().enumerate() {
            let Some(field) = types.struct_def(generator.frame).fields.get(index) else { continue };
            let local = body.local(param).ty;
            if field.ty != local && !matches!(types.kind(field.ty), TyKind::Ref { inner, .. } if *inner == local) {
                sink.emit(Diagnostic::error(
                    codes::E0900,
                    body.span,
                    format!(
                        "a generator whose parameter `{}` it holds as `{}` and reads as `{}` is not supported yet",
                        body.local(param).name.as_deref().unwrap_or("_"),
                        types.display(field.ty),
                        types.display(local)
                    ),
                ));
                errors += 1;
            }
        }
        let live_in = live_in(body);
        let pts = points_to(body, types, &callees);
        let mut reported: HashSet<(u8, Option<LocalId>)> = HashSet::new();
        let mut saved: BTreeSet<LocalId> = BTreeSet::new();
        for point in generator.points.iter().skip(1) {
            let live = live_across(&live_in, point);
            // `[CORO-9]` — an `unsafe gen fn` may keep a raw pointer into its own
            // frame, which its safety note answers for (`[UNS-7]`, `[UNS-8]`);
            // Safe code never makes a frame point into itself.
            let pinned = |ty: Ty| body.is_unsafe && matches!(types.kind(ty), TyKind::Ptr { .. });
            let mut holders: Vec<(Option<LocalId>, BTreeSet<Target>)> = live
                .iter()
                .filter(|&local| local != LocalId(1) && !pinned(body.local(local).ty))
                .map(|local| (Some(local), pts[local.0 as usize].clone()))
                .collect();
            if let Some(value) = &point.value
                && !matches!(value, Operand::Copy(place) | Operand::Move(place) if pinned(place_ty(body, types, place)))
            {
                holders.push((None, operand_targets(body, types, &pts, value)));
            }
            for (holder, targets) in holders {
                for target in targets {
                    match target {
                        Target::Local(local) if local != LocalId(1) => {
                            if !reported.insert((0, Some(local))) {
                                continue;
                            }
                            let what = match name(local) {
                                Some(name) => format!("`{name}`, a local of this generator,"),
                                None => "a local of this generator".to_string(),
                            };
                            let message = match holder {
                                None => format!("this `yield` gives out a borrow of {what} which outlives the suspension"),
                                Some(_) => format!("a borrow of {what} is held across this `yield`"),
                            };
                            sink.emit(
                                Diagnostic::error(codes::E2221, point.span, message)
                                    .primary_label("the generator suspends here")
                                    .note("a suspended frame is a value that may move, which would leave the borrow pointing where the local was [CORO-6]")
                                    .help("iterate the collection `owned` (`for x in owned xs:`), or by index; borrows of the generator's parameters may be held"),
                            );
                            errors += 1;
                        }
                        Target::ClassField => {
                            if !reported.insert((1, None)) {
                                continue;
                            }
                            sink.emit(
                                Diagnostic::error(codes::E2229, point.span, "a borrow of a field of a class object is held across this `yield`")
                                    .primary_label("the generator suspends here")
                                    .note("a long-term access to an object's field may not span a suspension, when any other code may run [CORO-12]")
                                    .help("copy what is needed out of the field before the `yield`, or iterate by index"),
                            );
                            errors += 1;
                        }
                        Target::Local(_) => {}
                    }
                }
            }
            for local in live.iter() {
                if local == LocalId(1) || !types.is_must_drop(body.local(local).ty) || !reported.insert((2, Some(local))) {
                    continue;
                }
                let what = name(local).map_or("a `@must_drop` value".to_string(), |name| format!("`{name}`, a `@must_drop` value,"));
                sink.emit(
                    Diagnostic::error(codes::E2231, point.span, format!("{what} is live across this `yield`"))
                        .primary_label("the generator suspends here")
                        .note("a suspended generator can be forgotten or leaked, so the value's `drop`, which a safety guarantee depends on, might never run [CORO-13]")
                        .help("close its scope before the `yield`"),
                );
                errors += 1;
            }
            saved.extend(live.iter());
        }
        // `[CORO-5]` — the frames this one holds by value.
        let mut held = Vec::new();
        for local in saved.iter().copied().filter(|local| local.0 as usize > body.arg_count) {
            frames_in(types, body.local(local).ty, &mut held, &mut HashSet::new());
        }
        holds.push((generator.frame, held, body.span));
    }
    // A frame holding itself, directly or through another's, has no size.
    for (frame, _, span) in &holds {
        let mut seen = HashSet::new();
        let mut stack: Vec<StructId> = holds.iter().filter(|(f, ..)| f == frame).flat_map(|(_, held, _)| held.clone()).collect();
        while let Some(next) = stack.pop() {
            if next == *frame {
                sink.emit(
                    Diagnostic::error(codes::E2200, *span, "this generator holds its own frame across a `yield`, which gives the frame no size")
                        .note("a frame's size is fixed when it is compiled [CORO-5]; a recursive generator holds the inner one in a `Box[dyn Iterator[Item = …]]`"),
                );
                errors += 1;
                break;
            }
            if seen.insert(next) {
                stack.extend(holds.iter().filter(|(f, ..)| *f == next).flat_map(|(_, held, _)| held.clone()));
            }
        }
    }
    errors
}

/// The generator frames a value of `ty` holds in its own storage.
fn frames_in(types: &TypeTable, ty: Ty, out: &mut Vec<StructId>, seen: &mut HashSet<Ty>) {
    if !seen.insert(ty) {
        return;
    }
    match types.kind(ty) {
        TyKind::Struct(id) => {
            if types.is_coroutine_frame(ty) {
                out.push(*id);
            }
            for field in &types.struct_def(*id).fields {
                frames_in(types, field.ty, out, seen);
            }
        }
        TyKind::Enum(id) => {
            for variant in &types.enum_def(*id).variants {
                for field in &variant.fields {
                    frames_in(types, field.ty, out, seen);
                }
            }
        }
        TyKind::Tuple(items) => items.iter().for_each(|&item| frames_in(types, item, out, seen)),
        TyKind::Array { elem, .. } => frames_in(types, *elem, out, seen),
        _ => {}
    }
}

fn builds_frame(stmt: &Stmt, types: &TypeTable) -> Option<StructId> {
    match &stmt.kind {
        StmtKind::Assign { rvalue: Rvalue::Aggregate { kind: AggregateKind::Struct(id), .. }, .. } if types.is_coroutine_frame_struct(*id) => {
            Some(*id)
        }
        _ => None,
    }
}

// -- the transform ------------------------------------------------------------

/// How a local of the generator's body is reached in `next` and `drop`.
#[derive(Clone)]
enum Home {
    /// A local of the new body.
    Local(LocalId),
    /// A place in the frame: `self` (local 1) and this projection.
    Frame(Vec<Projection>),
}

/// `[CORO-4]` — rewrite each generator's body into its frame's `next`,
/// `drop` and `result`, and each constructor's literal into the frame's
/// first state. Run after the borrow check: the bodies it makes are checked
/// by construction.
/// Returns whether there was any; the bodies it made have their callable
/// summaries made again (`[MIR-REG-1]`).
pub fn transform(bodies: &mut Vec<Body>, types: &mut TypeTable) -> bool {
    if !transform_all(bodies, types) {
        return false;
    }
    crate::install_callable_regions_all(bodies, types);
    true
}

fn transform_all(bodies: &mut Vec<Body>, types: &mut TypeTable) -> bool {
    let generators = find_generators(bodies, types);
    if generators.is_empty() {
        return false;
    }
    let index: HashMap<String, usize> = bodies.iter().enumerate().map(|(i, body)| (body.symbol.clone(), i)).collect();
    let callees: HashMap<&str, &Body> = bodies.iter().map(|body| (body.symbol.as_str(), body)).collect();
    let mut states: HashMap<StructId, u64> = HashMap::new();
    let mut replaced: Vec<(usize, Body)> = Vec::new();
    for generator in &generators {
        let body = &bodies[generator.body];
        let owner = types.symbol_name(generator.frame_ty);
        let next_symbol = method_symbol(&owner, "next");
        let drop_symbol = ember_branding::drop_function(&owner);
        let result_symbol = method_symbol(&owner, "result");
        let (Some(&next_index), Some(&drop_index)) = (index.get(&next_symbol), index.get(&drop_symbol)) else {
            panic!("coroutine transform: the frame `{owner}` has no `next` or `drop` body");
        };
        let pts = points_to(body, types, &callees);
        let layout = lay_out(generator, body, &pts, types);
        let next = resume_body(generator, body, &bodies[next_index], &layout, types);
        let drop = drop_body(generator, body, &bodies[drop_index], &layout, types);
        replaced.push((next_index, next));
        replaced.push((drop_index, drop));
        if let Some(&result_index) = index.get(&result_symbol)
            && let Some(result_field) = layout.result
        {
            replaced.push((result_index, result_body(&bodies[result_index], &layout, result_field, body.span, types)));
        }
        // The frame's fields: the parameters (already there), the locals
        // live across a suspension, the result, the state.
        let def = types.struct_def_mut(generator.frame);
        def.fields.extend(layout.added.iter().cloned());
        def.drops_fields = false;
        states.insert(generator.frame, layout.state_field as u64);
    }
    for (index, body) in replaced {
        bodies[index] = body;
    }
    // `[CORO-1]` — a constructor's literal is the frame's first state.
    for body in bodies.iter_mut() {
        for block in &mut body.blocks {
            let mut out = Vec::with_capacity(block.stmts.len());
            for stmt in block.stmts.drain(..) {
                let Some(frame) = builds_frame(&stmt, types) else {
                    out.push(stmt);
                    continue;
                };
                let StmtKind::Assign { place, rvalue: Rvalue::Aggregate { operands, .. } } = stmt.kind else { unreachable!() };
                for (field, operand) in operands.into_iter().enumerate() {
                    out.push(Stmt::new(StmtKind::Assign { place: place.clone().field(field), rvalue: Rvalue::Use(operand) }, stmt.span));
                }
                let state = states[&frame] as usize;
                let state_ty = types.struct_def(frame).fields[state].ty;
                out.push(Stmt::new(
                    StmtKind::Assign { place: place.clone().field(state), rvalue: Rvalue::Use(Operand::Const(Const::Int { value: 0, ty: state_ty })) },
                    stmt.span,
                ));
            }
            block.stmts = out;
        }
    }
    let gone: HashSet<usize> = generators.iter().map(|generator| generator.body).collect();
    let mut position = 0;
    bodies.retain(|_| {
        position += 1;
        !gone.contains(&(position - 1))
    });
    true
}

/// The generator a frame is the frame of, as its program names it: the
/// frame's name less `.Generator`.
fn generator_name(types: &TypeTable, frame: StructId) -> String {
    let def = types.struct_def(frame);
    let name = def.origin.as_ref().map_or(def.name, |(origin, _)| *origin).to_string();
    match name.rfind(".Generator") {
        Some(at) => name[..at].to_string(),
        None => name,
    }
}

/// `[MNG-1]` — a method's C symbol, as the checker makes it.
fn method_symbol(owner: &str, name: &str) -> String {
    let owner: String = owner.chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '_' }).collect();
    ember_branding::mangled(&format!("{}_{name}", owner.trim_matches('_')))
}

/// Where everything the frame holds lives.
struct Layout {
    /// Each body local's home in the new bodies' terms (`Home::Local` ids
    /// are assigned per new body; here only frame homes are fixed).
    frame_homes: HashMap<LocalId, Vec<Projection>>,
    /// The fields the transform adds, in order after the parameters'.
    added: Vec<FieldDef>,
    result: Option<usize>,
    state_field: usize,
    state_ty: Ty,
    done: i128,
    taken: i128,
    /// The parameters the frame holds a copy of that the body does not own
    /// (a borrowed class handle): released when the frame is done or dropped.
    released: Vec<usize>,
}

const C_KEYWORDS: &[&str] = &[
    "auto", "break", "case", "char", "const", "continue", "default", "do", "double", "else", "enum", "extern", "float", "for",
    "goto", "if", "inline", "int", "long", "register", "restrict", "return", "short", "signed", "sizeof", "static", "struct",
    "switch", "typedef", "union", "unsigned", "void", "volatile", "while", "bool", "true", "false",
];

fn lay_out(generator: &Generator, body: &Body, pts: &[BTreeSet<Target>], types: &mut TypeTable) -> Layout {
    let live_in = live_in(body);
    let mut saved = BTreeSet::new();
    for point in &generator.points {
        let live = live_across(&live_in, point);
        // `[CORO-9]` — what a pointer kept across the suspension points to
        // lives in the frame too (an `unsafe gen fn`'s own; Safe code keeps
        // none, `E2221`).
        for local in live.iter() {
            for target in &pts[local.0 as usize] {
                if let Target::Local(pointee) = target {
                    saved.insert(*pointee);
                }
            }
        }
        saved.extend(live.iter());
    }
    let params: HashSet<LocalId> = generator.params.iter().copied().collect();
    let mut saved: Vec<LocalId> = saved.into_iter().filter(|local| local.0 > 1 && !params.contains(local)).collect();
    // Widest first: no padding between them.
    saved.sort_by_key(|local| std::cmp::Reverse(types.layout(body.local(*local).ty).align));
    let def = types.struct_def(generator.frame);
    let base = def.fields.len();
    let mut names: HashSet<String> = def.fields.iter().map(|field| field.name.to_string()).collect();
    let span = body.span;
    let field = |name: &str, ty: Ty, names: &mut HashSet<String>| {
        let mut clean: String = name.chars().map(|c| if c.is_ascii_alphanumeric() || c == '_' { c } else { '_' }).collect();
        if clean.is_empty() || clean.starts_with(|c: char| c.is_ascii_digit()) || C_KEYWORDS.contains(&clean.as_str()) {
            clean = format!("_{clean}");
        }
        let mut unique = clean.clone();
        let mut n = 1;
        while !names.insert(unique.clone()) {
            n += 1;
            unique = format!("{clean}{n}");
        }
        FieldDef {
            name: Symbol::intern(&unique),
            ty,
            span,
            ty_span: span,
            has_default: false,
            is_let: false,
            read_only_outside: false,
            vis: FieldVis::Private,
        }
    };
    let mut frame_homes = HashMap::new();
    let mut added = Vec::new();
    for (index, &param) in generator.params.iter().enumerate() {
        let field_ty = types.struct_def(generator.frame).fields[index].ty;
        let local_ty = body.local(param).ty;
        let mut projection = vec![Projection::Deref, Projection::Field(index)];
        // Held as the `ref` the caller passed, read through it; `check_all`
        // refused any other difference.
        if field_ty != local_ty {
            projection.push(Projection::Deref);
        }
        frame_homes.insert(param, projection);
    }
    for &local in &saved {
        let decl = body.local(local);
        let name = decl.name.clone().unwrap_or_else(|| format!("_{}", local.0));
        added.push(field(&name, decl.ty, &mut names));
        frame_homes.insert(local, vec![Projection::Deref, Projection::Field(base + added.len() - 1)]);
    }
    let result_ty = body.return_ty();
    let result = (!matches!(types.kind(result_ty), TyKind::Void)).then(|| {
        added.push(field("result", result_ty, &mut names));
        base + added.len() - 1
    });
    // The states: 0 not started, one per `yield`, done, its result taken.
    let yields = generator.points.len() - 1;
    let done = yields as i128 + 1;
    let taken = done + 1;
    let state_ty = types.intern(TyKind::Uint(match taken {
        ..=0xFF => UintTy::U8,
        ..=0xFFFF => UintTy::U16,
        _ => UintTy::U32,
    }));
    added.push(field("state", state_ty, &mut names));
    let state_field = base + added.len() - 1;
    let released = generator
        .params
        .iter()
        .enumerate()
        .filter(|&(index, &param)| {
            body.borrowed_params.contains(&param)
                && types.needs_drop(types.struct_def(generator.frame).fields[index].ty)
                && !matches!(types.kind(types.struct_def(generator.frame).fields[index].ty), TyKind::Ref { .. })
        })
        .map(|(index, _)| index)
        .collect();
    Layout { frame_homes, added, result, state_field, state_ty, done, taken, released }
}

/// Builds one of the new bodies from the generator's: its locals reached
/// through `homes`, its blocks copied as they are reached.
struct Builder<'a> {
    body: &'a Body,
    homes: Vec<Home>,
    locals: Vec<LocalDecl>,
    blocks: Vec<BasicBlock>,
    block_map: HashMap<usize, usize>,
    pending: Vec<usize>,
}

impl<'a> Builder<'a> {
    fn new(body: &'a Body, stub: &Body, layout: &Layout) -> Builder<'a> {
        let mut locals: Vec<LocalDecl> = stub.locals[..2].to_vec();
        let mut homes = Vec::with_capacity(body.locals.len());
        for (index, decl) in body.locals.iter().enumerate() {
            let local = LocalId(index as u32);
            let home = if index == 1 {
                Home::Local(LocalId(1))
            } else if let Some(projection) = layout.frame_homes.get(&local) {
                Home::Frame(projection.clone())
            } else {
                let id = LocalId(locals.len() as u32);
                let kind = if index == 0 { LocalKind::Temp } else if decl.kind == LocalKind::Arg { LocalKind::User } else { decl.kind };
                locals.push(LocalDecl { ty: decl.ty, kind, name: decl.name.clone(), span: decl.span });
                Home::Local(id)
            };
            homes.push(home);
        }
        Builder { body, homes, locals, blocks: Vec::new(), block_map: HashMap::new(), pending: Vec::new() }
    }

    fn frame(&self, projection: Vec<Projection>) -> Place {
        Place { local: LocalId(1), projection }
    }

    fn temp(&mut self, ty: Ty) -> LocalId {
        let id = LocalId(self.locals.len() as u32);
        self.locals.push(LocalDecl { ty, kind: LocalKind::Temp, name: None, span: self.body.span });
        id
    }

    /// A local's value in a local of the new body, loaded first if it lives
    /// in the frame (an index, a drop flag).
    fn local_value(&mut self, local: LocalId, span: Span, pre: &mut Vec<Stmt>) -> LocalId {
        match self.homes[local.0 as usize].clone() {
            Home::Local(id) => id,
            Home::Frame(projection) => {
                let ty = self.body.local(local).ty;
                let temp = self.temp(ty);
                pre.push(Stmt::new(StmtKind::Assign { place: Place::local(temp), rvalue: Rvalue::Use(Operand::Copy(self.frame(projection))) }, span));
                temp
            }
        }
    }

    fn place(&mut self, place: &Place, span: Span, pre: &mut Vec<Stmt>) -> Place {
        let mut projection = Vec::with_capacity(place.projection.len());
        for step in &place.projection {
            projection.push(match step {
                Projection::Index(index) => Projection::Index(self.local_value(*index, span, pre)),
                other => other.clone(),
            });
        }
        match self.homes[place.local.0 as usize].clone() {
            Home::Local(id) => Place { local: id, projection },
            Home::Frame(mut prefix) => {
                prefix.extend(projection);
                self.frame(prefix)
            }
        }
    }

    fn operand(&mut self, operand: &Operand, span: Span, pre: &mut Vec<Stmt>) -> Operand {
        match operand {
            Operand::Copy(place) => Operand::Copy(self.place(place, span, pre)),
            Operand::Move(place) => Operand::Move(self.place(place, span, pre)),
            Operand::Const(value) => Operand::Const(value.clone()),
        }
    }

    fn rvalue(&mut self, rvalue: &Rvalue, span: Span, pre: &mut Vec<Stmt>) -> Rvalue {
        match rvalue {
            Rvalue::Use(operand) => Rvalue::Use(self.operand(operand, span, pre)),
            Rvalue::BinaryOp { op, lhs, rhs } => Rvalue::BinaryOp { op: *op, lhs: self.operand(lhs, span, pre), rhs: self.operand(rhs, span, pre) },
            Rvalue::UnaryOp { op, operand } => Rvalue::UnaryOp { op: *op, operand: self.operand(operand, span, pre) },
            Rvalue::Cast { kind, operand, to } => Rvalue::Cast { kind: kind.clone(), operand: self.operand(operand, span, pre), to: *to },
            Rvalue::Aggregate { kind, operands } => {
                Rvalue::Aggregate { kind: *kind, operands: operands.iter().map(|operand| self.operand(operand, span, pre)).collect() }
            }
            Rvalue::Repeat { value, count } => Rvalue::Repeat { value: self.operand(value, span, pre), count: *count },
            Rvalue::Discriminant(place) => Rvalue::Discriminant(self.place(place, span, pre)),
            Rvalue::Ref { place, mutable } => Rvalue::Ref { place: self.place(place, span, pre), mutable: *mutable },
        }
    }

    fn stmt(&mut self, stmt: &Stmt, out: &mut Vec<Stmt>) {
        let span = stmt.span;
        let mut pre = Vec::new();
        let kind = match &stmt.kind {
            StmtKind::Assign { place, rvalue } => {
                let rvalue = self.rvalue(rvalue, span, &mut pre);
                StmtKind::Assign { place: self.place(place, span, &mut pre), rvalue }
            }
            StmtKind::BeginAccess { place, mutable } => StmtKind::BeginAccess { place: self.place(place, span, &mut pre), mutable: *mutable },
            StmtKind::BeginAccessTransfer { place, mutable } => {
                StmtKind::BeginAccessTransfer { place: self.place(place, span, &mut pre), mutable: *mutable }
            }
            StmtKind::EndAccess { place, mutable } => StmtKind::EndAccess { place: self.place(place, span, &mut pre), mutable: *mutable },
            StmtKind::EndAccessTransfer { place, mutable } => {
                StmtKind::EndAccessTransfer { place: self.place(place, span, &mut pre), mutable: *mutable }
            }
            StmtKind::CheckedBinaryOp { dest, overflow, op, lhs, rhs } => {
                let lhs = self.operand(lhs, span, &mut pre);
                let rhs = self.operand(rhs, span, &mut pre);
                StmtKind::CheckedBinaryOp { dest: self.place(dest, span, &mut pre), overflow: self.place(overflow, span, &mut pre), op: *op, lhs, rhs }
            }
            StmtKind::StorageLive(local) | StmtKind::StorageDead(local) => match self.homes[local.0 as usize] {
                Home::Local(id) => match stmt.kind {
                    StmtKind::StorageLive(_) => StmtKind::StorageLive(id),
                    _ => StmtKind::StorageDead(id),
                },
                // The frame's storage lasts as long as the frame.
                Home::Frame(_) => StmtKind::Nop,
            },
            StmtKind::Drop { place, flag, scope_end } => {
                let flag = flag.map(|flag| self.local_value(flag, span, &mut pre));
                StmtKind::Drop { place: self.place(place, span, &mut pre), flag, scope_end: *scope_end }
            }
            StmtKind::Nop => StmtKind::Nop,
        };
        out.extend(pre);
        out.push(Stmt::new(kind, span));
    }

    /// The new block a block of the body becomes, made when first reached.
    fn target(&mut self, block: usize) -> BasicBlockId {
        if let Some(&id) = self.block_map.get(&block) {
            return BasicBlockId(id as u32);
        }
        let id = self.new_block(Terminator::Unreachable, self.body.blocks[block].terminator_span);
        self.block_map.insert(block, id);
        self.pending.push(block);
        BasicBlockId(id as u32)
    }

    fn new_block(&mut self, terminator: Terminator, span: Span) -> usize {
        self.blocks.push(BasicBlock { stmts: Vec::new(), terminator, terminator_span: span });
        self.blocks.len() - 1
    }

    fn frame_field(&self, field: usize) -> Place {
        self.frame(vec![Projection::Deref, Projection::Field(field)])
    }

    /// Copy every reached block, `rewrite` deciding each terminator.
    fn copy_blocks(&mut self, rewrite: &mut dyn FnMut(&mut Builder<'a>, usize, &mut Vec<Stmt>) -> Option<Terminator>) {
        let body = self.body;
        while let Some(old) = self.pending.pop() {
            let new = self.block_map[&old];
            let source = &body.blocks[old];
            let mut stmts = Vec::new();
            for stmt in &source.stmts {
                self.stmt(stmt, &mut stmts);
            }
            let terminator = match rewrite(self, old, &mut stmts) {
                Some(terminator) => terminator,
                None => self.terminator(&source.terminator, source.terminator_span, &mut stmts),
            };
            self.blocks[new].stmts = stmts;
            self.blocks[new].terminator = terminator;
        }
    }

    fn terminator(&mut self, terminator: &Terminator, span: Span, pre: &mut Vec<Stmt>) -> Terminator {
        match terminator {
            Terminator::Goto(target) => Terminator::Goto(self.target(target.0 as usize)),
            Terminator::SwitchInt { discr, targets, otherwise } => {
                let discr = self.operand(discr, span, pre);
                let targets = targets.iter().map(|(value, target)| (*value, self.target(target.0 as usize))).collect();
                Terminator::SwitchInt { discr, targets, otherwise: self.target(otherwise.0 as usize) }
            }
            Terminator::Return => Terminator::Return,
            Terminator::Unreachable => Terminator::Unreachable,
            Terminator::Call { func, args, dest, next } => {
                // A `yield` as a `void` expression: nothing to do once resumed.
                if let FuncRef::Direct { symbol, .. } = func
                    && *symbol == resumed_symbol()
                {
                    return Terminator::Goto(self.target(next.0 as usize));
                }
                let func = match func {
                    FuncRef::Indirect { operand, ty, latebound, sources } => {
                        FuncRef::Indirect { operand: self.operand(operand, span, pre), ty: *ty, latebound: *latebound, sources: sources.clone() }
                    }
                    other => other.clone(),
                };
                let args = args.iter().map(|arg| self.operand(arg, span, pre)).collect();
                let dest = self.place(dest, span, pre);
                Terminator::Call { func, args, dest, next: self.target(next.0 as usize) }
            }
            Terminator::Assert { cond, expected, msg, next, span: checked } => {
                let cond = self.operand(cond, span, pre);
                let msg = match msg {
                    AssertKind::Bounds { len, index } => AssertKind::Bounds { len: self.operand(len, span, pre), index: self.operand(index, span, pre) },
                    AssertKind::RefCellBorrow { file, line } => {
                        AssertKind::RefCellBorrow { file: self.operand(file, span, pre), line: self.operand(line, span, pre) }
                    }
                    AssertKind::Panic { message } => AssertKind::Panic { message: self.operand(message, span, pre) },
                    other => other.clone(),
                };
                Terminator::Assert { cond, expected: *expected, msg, next: self.target(next.0 as usize), span: *checked }
            }
        }
    }

    fn assign(&self, place: Place, rvalue: Rvalue, span: Span) -> Stmt {
        Stmt::new(StmtKind::Assign { place, rvalue }, span)
    }

    /// Release what the frame holds of a parameter the body does not own.
    fn release(&self, layout: &Layout, span: Span, out: &mut Vec<Stmt>) {
        for &field in &layout.released {
            out.push(Stmt::new(StmtKind::Drop { place: self.frame_field(field), flag: None, scope_end: true }, span));
        }
    }

    fn finish(self, stub: &Body, from: &Body, entry: Vec<Stmt>, entry_terminator: Terminator) -> Body {
        let mut blocks = self.blocks;
        blocks[0].stmts = entry;
        blocks[0].terminator = entry_terminator;
        let mut made = stub.clone();
        made.locals = self.locals;
        made.blocks = blocks;
        made.overflow = from.overflow;
        made.fp = from.fp;
        made.callable_regions = None;
        made.call_argument_bindings = Vec::new();
        made.for_iterators = Vec::new();
        made.borrowed_params = Vec::new();
        // The switch goes straight to where each state resumes.
        let through = |blocks: &[BasicBlock], mut target: BasicBlockId| {
            for _ in 0..blocks.len() {
                match &blocks[target.0 as usize] {
                    BasicBlock { stmts, terminator: Terminator::Goto(next), .. } if stmts.iter().all(|stmt| matches!(stmt.kind, StmtKind::Nop)) => target = *next,
                    _ => break,
                }
            }
            target
        };
        if let Terminator::SwitchInt { targets, otherwise, .. } = made.blocks[0].terminator.clone() {
            let targets = targets.into_iter().map(|(value, target)| (value, through(&made.blocks, target))).collect();
            let otherwise = through(&made.blocks, otherwise);
            if let Terminator::SwitchInt { targets: old, otherwise: old_otherwise, .. } = &mut made.blocks[0].terminator {
                *old = targets;
                *old_otherwise = otherwise;
            }
        }
        prune(&mut made);
        drop_unused_locals(&mut made);
        made
    }
}

fn option_variant(types: &TypeTable, option: Ty, name: &str) -> (ember_types::EnumId, usize) {
    let TyKind::Enum(id) = *types.kind(option) else { panic!("coroutine transform: `next` does not return an `Option`") };
    let (index, _) = types.enum_def(id).variant(Symbol::intern(name)).expect("an `Option` has `Some` and `None`");
    (id, index)
}

fn state_const(layout: &Layout, value: i128) -> Operand {
    Operand::Const(Const::Int { value: value as u128, ty: layout.state_ty })
}

/// `next`: switch on the state to where the body was left, and run it to the
/// next `yield` or its end.
fn resume_body(generator: &Generator, body: &Body, stub: &Body, layout: &Layout, types: &TypeTable) -> Body {
    let mut builder = Builder::new(body, stub, layout);
    let span = body.span;
    let option = stub.return_ty();
    let (option_id, some) = option_variant(types, option, "Some");
    let (_, none) = option_variant(types, option, "None");
    let entry = builder.new_block(Terminator::Unreachable, span);
    let mut targets = vec![(0, builder.target(0))];
    for (state, point) in generator.points.iter().enumerate().skip(1) {
        targets.push((state as i128, builder.target(point.next)));
    }
    let finished = builder.new_block(Terminator::Return, span);
    let nothing = builder.assign(Place::local(RETURN_LOCAL), Rvalue::Aggregate { kind: AggregateKind::Enum(option_id, none), operands: Vec::new() }, span);
    builder.blocks[finished].stmts.push(nothing);
    targets.push((layout.done, BasicBlockId(finished as u32)));
    targets.push((layout.taken, BasicBlockId(finished as u32)));
    let impossible = builder.new_block(Terminator::Unreachable, span);
    let dispatch = Terminator::SwitchInt {
        discr: Operand::Copy(builder.frame_field(layout.state_field)),
        targets,
        otherwise: BasicBlockId(impossible as u32),
    };
    let calls: HashMap<usize, usize> = generator.points.iter().enumerate().map(|(state, point)| (point.call, state)).collect();
    let switches: HashMap<usize, usize> = generator.points.iter().map(|point| (point.switch, point.resume)).collect();
    let ret = builder.homes[0].clone();
    builder.copy_blocks(&mut |builder, old, stmts| {
        let source = &body.blocks[old];
        let span = source.terminator_span;
        if let Some(&state) = calls.get(&old) {
            let point = &generator.points[state];
            if state == 0 {
                return Some(Terminator::Goto(builder.target(point.next)));
            }
            // `[CORO-2]` — suspend: keep where, give out the value.
            let value = point.value.as_ref().map(|value| builder.operand(value, span, stmts)).unwrap_or(Operand::Const(Const::Void));
            stmts.push(builder.assign(Place::local(RETURN_LOCAL), Rvalue::Aggregate { kind: AggregateKind::Enum(option_id, some), operands: vec![value] }, span));
            stmts.push(builder.assign(builder.frame_field(layout.state_field), Rvalue::Use(state_const(layout, state as i128)), span));
            return Some(Terminator::Return);
        }
        if let Some(&resume) = switches.get(&old) {
            return Some(Terminator::Goto(builder.target(resume)));
        }
        if matches!(source.terminator, Terminator::Return) {
            // `[CORO-3]` — the end: keep the result, release what the frame
            // holds of its parameters, and say there is nothing more.
            if let (Some(field), Home::Local(ret)) = (layout.result, &ret) {
                stmts.push(builder.assign(builder.frame_field(field), Rvalue::Use(Operand::Move(Place::local(*ret))), span));
            }
            builder.release(layout, span, stmts);
            stmts.push(builder.assign(builder.frame_field(layout.state_field), Rvalue::Use(state_const(layout, layout.done)), span));
            stmts.push(builder.assign(Place::local(RETURN_LOCAL), Rvalue::Aggregate { kind: AggregateKind::Enum(option_id, none), operands: Vec::new() }, span));
            return Some(Terminator::Return);
        }
        None
    });
    let _ = entry;
    let mut made = builder.finish(stub, body, Vec::new(), dispatch);
    // What a diagnostic about the body calls it: the generator.
    made.name = generator_name(types, generator.frame);
    made.is_unsafe = body.is_unsafe;
    made.inline = body.inline;
    made.static_safe = body.static_safe;
    made.contracts = body.contracts;
    made
}

/// `[CORO-7]` — the frame's `drop`: switch on the state to the dropped path
/// of the suspension it was left at.
fn drop_body(generator: &Generator, body: &Body, stub: &Body, layout: &Layout, types: &TypeTable) -> Body {
    let mut builder = Builder::new(body, stub, layout);
    let span = body.span;
    builder.new_block(Terminator::Unreachable, span);
    let mut targets = Vec::new();
    for (state, point) in generator.points.iter().enumerate() {
        targets.push((state as i128, builder.target(point.next)));
    }
    let finished = builder.new_block(Terminator::Return, span);
    if let Some(field) = layout.result
        && types.needs_drop(types.struct_def(generator.frame).fields.get(field).map_or(body.return_ty(), |field| field.ty))
    {
        let drop = StmtKind::Drop { place: builder.frame_field(field), flag: None, scope_end: true };
        builder.blocks[finished].stmts.push(Stmt::new(drop, span));
    }
    targets.push((layout.done, BasicBlockId(finished as u32)));
    let ended = builder.new_block(Terminator::Return, span);
    let dispatch = Terminator::SwitchInt {
        discr: Operand::Copy(builder.frame_field(layout.state_field)),
        targets,
        otherwise: BasicBlockId(ended as u32),
    };
    let switches: HashMap<usize, usize> = generator.points.iter().map(|point| (point.switch, point.dropped)).collect();
    builder.copy_blocks(&mut |builder, old, stmts| {
        let source = &body.blocks[old];
        if let Some(&dropped) = switches.get(&old) {
            return Some(Terminator::Goto(builder.target(dropped)));
        }
        if matches!(source.terminator, Terminator::Return) {
            builder.release(layout, source.terminator_span, stmts);
            return Some(Terminator::Return);
        }
        None
    });
    builder.finish(stub, body, Vec::new(), dispatch)
}

/// `result()`: the returned value, once, after the end.
fn result_body(stub: &Body, layout: &Layout, field: usize, span: Span, types: &TypeTable) -> Body {
    let option = stub.return_ty();
    let (option_id, some) = option_variant(types, option, "Some");
    let (_, none) = option_variant(types, option, "None");
    let frame = |field: usize| Place { local: LocalId(1), projection: vec![Projection::Deref, Projection::Field(field)] };
    let blocks = vec![
        BasicBlock {
            stmts: Vec::new(),
            terminator: Terminator::SwitchInt {
                discr: Operand::Copy(frame(layout.state_field)),
                targets: vec![(layout.done, BasicBlockId(1))],
                otherwise: BasicBlockId(2),
            },
            terminator_span: span,
        },
        BasicBlock {
            stmts: vec![
                Stmt::new(StmtKind::Assign { place: frame(layout.state_field), rvalue: Rvalue::Use(state_const(layout, layout.taken)) }, span),
                Stmt::new(
                    StmtKind::Assign {
                        place: Place::local(RETURN_LOCAL),
                        rvalue: Rvalue::Aggregate { kind: AggregateKind::Enum(option_id, some), operands: vec![Operand::Move(frame(field))] },
                    },
                    span,
                ),
            ],
            terminator: Terminator::Return,
            terminator_span: span,
        },
        BasicBlock {
            stmts: vec![Stmt::new(
                StmtKind::Assign { place: Place::local(RETURN_LOCAL), rvalue: Rvalue::Aggregate { kind: AggregateKind::Enum(option_id, none), operands: Vec::new() } },
                span,
            )],
            terminator: Terminator::Return,
            terminator_span: span,
        },
    ];
    let mut made = stub.clone();
    made.locals.truncate(2);
    made.blocks = blocks;
    made.callable_regions = None;
    made
}

/// Each local a body names, `storage` telling a `StorageLive`/`StorageDead`'s
/// apart, given to `f` to read or rename.
fn visit_locals(body: &mut Body, f: &mut dyn FnMut(&mut LocalId, bool)) {
    fn place(place: &mut Place, f: &mut dyn FnMut(&mut LocalId, bool)) {
        f(&mut place.local, false);
        for projection in &mut place.projection {
            if let Projection::Index(index) = projection {
                f(index, false);
            }
        }
    }
    fn operand(operand: &mut Operand, f: &mut dyn FnMut(&mut LocalId, bool)) {
        if let Operand::Copy(p) | Operand::Move(p) = operand {
            place(p, f);
        }
    }
    for block in &mut body.blocks {
        for stmt in &mut block.stmts {
            match &mut stmt.kind {
                StmtKind::Assign { place: target, rvalue } => {
                    place(target, f);
                    match rvalue {
                        Rvalue::Use(o) | Rvalue::UnaryOp { operand: o, .. } | Rvalue::Cast { operand: o, .. } | Rvalue::Repeat { value: o, .. } => {
                            operand(o, f)
                        }
                        Rvalue::BinaryOp { lhs, rhs, .. } => {
                            operand(lhs, f);
                            operand(rhs, f);
                        }
                        Rvalue::Aggregate { operands, .. } => operands.iter_mut().for_each(|o| operand(o, f)),
                        Rvalue::Discriminant(p) | Rvalue::Ref { place: p, .. } => place(p, f),
                    }
                }
                StmtKind::BeginAccess { place: p, .. }
                | StmtKind::BeginAccessTransfer { place: p, .. }
                | StmtKind::EndAccess { place: p, .. }
                | StmtKind::EndAccessTransfer { place: p, .. } => place(p, f),
                StmtKind::CheckedBinaryOp { dest, overflow, lhs, rhs, .. } => {
                    place(dest, f);
                    place(overflow, f);
                    operand(lhs, f);
                    operand(rhs, f);
                }
                StmtKind::StorageLive(local) | StmtKind::StorageDead(local) => f(local, true),
                StmtKind::Drop { place: p, flag, .. } => {
                    place(p, f);
                    if let Some(flag) = flag {
                        f(flag, false);
                    }
                }
                StmtKind::Nop => {}
            }
        }
        match &mut block.terminator {
            Terminator::SwitchInt { discr, .. } => operand(discr, f),
            Terminator::Call { func, args, dest, .. } => {
                if let FuncRef::Indirect { operand: o, .. } = func {
                    operand(o, f);
                }
                args.iter_mut().for_each(|o| operand(o, f));
                place(dest, f);
            }
            Terminator::Assert { cond, msg, .. } => {
                operand(cond, f);
                match msg {
                    AssertKind::Bounds { len, index } => {
                        operand(len, f);
                        operand(index, f);
                    }
                    AssertKind::RefCellBorrow { file, line } => {
                        operand(file, f);
                        operand(line, f);
                    }
                    AssertKind::Panic { message } => operand(message, f),
                    _ => {}
                }
            }
            Terminator::Goto(_) | Terminator::Return | Terminator::Unreachable => {}
        }
    }
}

/// Drop the locals of the generator's body this one never uses (the
/// answers of the suspension calls, what the other half of the body uses),
/// so the C declares what it reads.
fn drop_unused_locals(body: &mut Body) {
    let mut used = vec![false; body.locals.len()];
    for used in used.iter_mut().take(body.arg_count + 1) {
        *used = true;
    }
    visit_locals(body, &mut |local, storage| {
        if !storage {
            used[local.0 as usize] = true;
        }
    });
    let mut renumber = vec![LocalId(0); body.locals.len()];
    let mut kept = Vec::new();
    for (index, decl) in std::mem::take(&mut body.locals).into_iter().enumerate() {
        if used[index] {
            renumber[index] = LocalId(kept.len() as u32);
            kept.push(decl);
        }
    }
    body.locals = kept;
    for block in &mut body.blocks {
        for stmt in &mut block.stmts {
            if let StmtKind::StorageLive(local) | StmtKind::StorageDead(local) = stmt.kind
                && !used[local.0 as usize]
            {
                stmt.kind = StmtKind::Nop;
            }
        }
    }
    visit_locals(body, &mut |local, _| *local = renumber[local.0 as usize]);
}

/// Drop the blocks nothing reaches from the entry, renumbering the rest.
fn prune(body: &mut Body) {
    let mut reached = vec![false; body.blocks.len()];
    let mut stack = vec![0];
    while let Some(block) = stack.pop() {
        if std::mem::replace(&mut reached[block], true) {
            continue;
        }
        stack.extend(successors(&body.blocks[block].terminator));
    }
    let mut renumber = vec![0u32; body.blocks.len()];
    let mut count = 0;
    for (index, &kept) in reached.iter().enumerate() {
        if kept {
            renumber[index] = count;
            count += 1;
        }
    }
    let fix = |target: &mut BasicBlockId| target.0 = renumber[target.0 as usize];
    let blocks = std::mem::take(&mut body.blocks);
    for (index, mut block) in blocks.into_iter().enumerate() {
        if !reached[index] {
            continue;
        }
        match &mut block.terminator {
            Terminator::Goto(target) => fix(target),
            Terminator::SwitchInt { targets, otherwise, .. } => {
                targets.iter_mut().for_each(|(_, target)| fix(target));
                fix(otherwise);
            }
            Terminator::Call { next, .. } | Terminator::Assert { next, .. } => fix(next),
            Terminator::Return | Terminator::Unreachable => {}
        }
        body.blocks.push(block);
    }
}
