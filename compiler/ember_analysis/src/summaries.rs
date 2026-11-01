//! `[EFF-24]` (A4-2, the first slice) — what each function may change, read and end, and through which
//! chain of calls, worked out from its code over the whole program's calls (`targets.rs`).
//!
//! A fact names a place a parameter reaches (`this` is a method's first): the parameter, then the
//! steps from it, at least one of them through a reference or an object's handle, since a change of
//! a parameter's own copy is no change its caller can see. A fact covers everything inside its
//! place. A body's own lines give its facts; a call gives its callee's, mapped onto the arguments,
//! for a call through a function value or a `dyn` or `virtual` method those of every function that
//! may run there (`[EFF-2]`, `[DSP-5]`, `[CLS-4]`); a `drop` ends what it drops. Whatever the
//! analysis cannot follow (a pointer read through a raw pointer, the result of a function it has no
//! body of) may be anything a parameter reaches, so nothing is missed: a fact too many refuses a
//! correct program later, a fact missing would accept a wrong one. Paths are cut at `MAX_PATH`
//! steps, a shorter path covering the longer ones, so recursion through a chain of links ends.
//!
//! Nothing is refused by it yet (M2): its first consumer is M1's `this`, whose change of its object
//! is worked out from its body (`[FN-4]`, `[EXC-15]`). `ember build --emit mir` prints it.

use std::collections::{HashMap, HashSet};

use ember_mir::{Body, Builtin, FuncRef, LocalId, Operand, Place, Projection, Rvalue, SoAOp, StmtKind, Terminator};
use ember_span::Span;
use ember_types::{TyKind, TypeTable};

use crate::regions::{place_type, project_type};
use crate::targets::Targets;

/// A path longer than this is cut to its first `MAX_PATH` steps, which cover it.
const MAX_PATH: usize = 8;

/// One step from a place to a place inside it, or to what it points at.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Step {
    Field(usize),
    /// Any element of a list or an array.
    Elements,
    Variant(usize),
    Column(usize),
    /// Through a reference, a pointer or an object's handle.
    Deref,
}

/// Where a fact's place starts: a parameter (0-based; a method's `this` is 0), or a `static`, the
/// same place in every function (SD1's `Static(symbol, path)`).
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Root {
    Param(usize),
    /// The static's name and the type of its value.
    Static(String, ember_types::Ty),
}

/// A place a parameter reaches, or a place inside a static.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Reach {
    pub root: Root,
    pub path: Vec<Step>,
}

/// A fact and the chain of calls that makes it: each call's callee (a body's index) and the call's
/// span; empty for the function's own line.
#[derive(Clone, Debug)]
pub struct Fact {
    pub reach: Reach,
    pub chain: Vec<(usize, Span)>,
}

/// `[EFF-24]` — what a function may change, read, reshape and end.
#[derive(Clone, Debug, Default)]
pub struct Summary {
    pub changes: Vec<Fact>,
    pub reads: Vec<Fact>,
    /// A change that may move or free memory (§VII.1's **reshape**): replacing a value that owns
    /// memory or changing an enum's variant, a list's or a string's elements added, removed,
    /// reordered or replaced. Each is a change too.
    pub reshapes: Vec<Fact>,
    /// A list or string it adds to the end of (`push`, `extend`): each is a reshape too.
    pub adds: Vec<Fact>,
    /// Every other reshape: elements removed, reordered or replaced, or the value replaced, which
    /// may move or free what is already there.
    pub removes: Vec<Fact>,
    pub ends: Vec<Fact>,
}

impl Summary {
    /// Whether it may change the place `param`'s `path` reaches, or anything inside it or holding it.
    pub fn may_change(&self, param: usize, path: &[Step]) -> bool {
        self.changes.iter().any(|fact| fact.reach.root == Root::Param(param) && overlaps(&fact.reach.path, path))
    }

    /// Whether it may change anything parameter `param` reaches: for a method's `this`, whether it
    /// changes its object.
    pub fn changes_param(&self, param: usize) -> bool {
        self.changes.iter().any(|fact| fact.reach.root == Root::Param(param))
    }

    /// Whether it may move or free memory inside, or holding, the place `param`'s `path` reaches.
    pub fn may_reshape(&self, param: usize, path: &[Step]) -> bool {
        self.reshapes.iter().any(|fact| fact.reach.root == Root::Param(param) && overlaps(&fact.reach.path, path))
    }

    /// `[EFF-24]` — whether all it does to the list at `reach` is add to its end (`push`, `extend`),
    /// reading or writing no element already in it: it adds there, every reshape there is an add,
    /// and nothing reads, changes or ends an element of it, or ends or reshapes what holds it.
    pub fn only_adds(&self, reach: &Reach) -> bool {
        let at = |fact: &Fact| fact.reach.root == reach.root;
        let element = |fact: &Fact| at(fact) && fact.reach.path.len() > reach.path.len() && fact.reach.path.starts_with(&reach.path) && fact.reach.path[reach.path.len()] == Step::Elements;
        self.adds.iter().any(|fact| fact.reach == *reach)
            && !self.removes.iter().any(|fact| at(fact) && overlaps(&fact.reach.path, &reach.path))
            && !self.reads.iter().chain(&self.changes).chain(&self.ends).any(element)
            && !self.ends.iter().any(|fact| at(fact) && reach.path.starts_with(&fact.reach.path))
    }
}

/// Whether one path lies inside the other.
fn overlaps(a: &[Step], b: &[Step]) -> bool {
    a.iter().zip(b).all(|(x, y)| x == y || matches!((x, y), (Step::Elements, _) | (_, Step::Elements)))
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum What {
    Change,
    Read,
    Reshape,
    Add,
    Remove,
    End,
}

/// Where a place is, in a body's own terms: a local and the steps from it.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Spot {
    local: LocalId,
    steps: Vec<Step>,
}

/// What a local holding a pointer may point at.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum Target {
    /// `*local` is this place.
    At(Spot),
    /// `*local` is this static's value.
    Static(String, ember_types::Ty),
    /// Not followed: anything any parameter reaches.
    Anything,
}

/// `[EFF-24]` — every body's summary, by index.
pub fn summarize(bodies: &[Body], types: &TypeTable) -> Vec<Summary> {
    let targets = Targets::new(bodies, types);
    let index: HashMap<&str, usize> = bodies.iter().enumerate().map(|(at, body)| (body.symbol.as_str(), at)).collect();
    let pointers: Vec<HashMap<LocalId, HashSet<Target>>> = bodies.iter().map(|body| pointers_of(body, types)).collect();
    let mut summaries: Vec<Summary> = vec![Summary::default(); bodies.len()];
    let mut known: Vec<HashSet<(What, Reach)>> = vec![HashSet::new(); bodies.len()];
    // Each round reads what the last one found; facts only grow and are finitely many (paths are cut),
    // so the rounds end.
    loop {
        let mut grew = false;
        for (at, body) in bodies.iter().enumerate() {
            let found = body_facts(at, body, bodies, types, &index, &targets, &pointers[at], &summaries);
            for (what, fact) in found {
                if known[at].insert((what, fact.reach.clone())) {
                    grew = true;
                    let list = match what {
                        What::Change => &mut summaries[at].changes,
                        What::Read => &mut summaries[at].reads,
                        What::Reshape => &mut summaries[at].reshapes,
                        What::Add => &mut summaries[at].adds,
                        What::Remove => &mut summaries[at].removes,
                        What::End => &mut summaries[at].ends,
                    };
                    list.push(fact);
                }
            }
        }
        if !grew {
            break;
        }
    }
    for summary in &mut summaries {
        for list in [&mut summary.changes, &mut summary.reads, &mut summary.reshapes, &mut summary.adds, &mut summary.removes, &mut summary.ends] {
            list.sort_by(|a, b| a.reach.cmp(&b.reach));
        }
    }
    summaries
}

/// The steps of a place. MIR reaches a class object's field straight through its handle (`n.next`
/// of a `n: Node`): that field is through the handle, a `Deref` the steps write.
fn steps_of(body: &Body, types: &TypeTable, place: &Place) -> Vec<Step> {
    let mut ty = body.local(place.local).ty;
    let mut variant = None;
    let mut steps = Vec::with_capacity(place.projection.len());
    for projection in &place.projection {
        if matches!(projection, Projection::Field(_)) && matches!(types.kind(ty), TyKind::Class(_)) {
            steps.push(Step::Deref);
        }
        steps.push(match projection {
            Projection::Field(field) => Step::Field(*field),
            Projection::Index(_) | Projection::ConstIndex(_) => Step::Elements,
            Projection::Downcast(variant) => Step::Variant(*variant),
            Projection::Column(column) => Step::Column(*column),
            Projection::Deref => Step::Deref,
        });
        ty = project_type(types, ty, projection, &mut variant);
    }
    steps
}

/// §VII.1 — whether writing a value of this type is a reshape: it owns memory the write may free or
/// move (a list, a string, a box, an object's handle, a `dyn` value, or anything holding one), or it
/// is an enum whose variant the write may change. A number, a `bool`, a view or a pointer is not.
fn reshaped_by_write(types: &TypeTable, ty: ember_types::Ty, depth: usize) -> bool {
    if depth > 8 {
        return true;
    }
    match types.kind(ty) {
        TyKind::Vec { .. } | TyKind::Class(_) | TyKind::Dyn { .. } | TyKind::ClassInterface(_) => true,
        TyKind::Struct(id) => types.struct_def(*id).fields.iter().any(|field| reshaped_by_write(types, field.ty, depth + 1)),
        TyKind::Enum(id) => types.enum_def(*id).variants.iter().any(|variant| !variant.fields.is_empty()),
        TyKind::Tuple(items) => items.iter().any(|item| reshaped_by_write(types, *item, depth + 1)),
        TyKind::Array { elem, .. } | TyKind::ArrayN { elem, .. } => reshaped_by_write(types, *elem, depth + 1),
        _ => false,
    }
}

/// §VII.1 — a built-in that adds, removes, reorders or replaces the elements of the list, string or
/// table its first argument points at, or replaces the value it points at.
fn reshaping_builtin(types: &TypeTable, which: &Builtin) -> bool {
    match which {
        Builtin::ArrayPush
        | Builtin::ArrayInsert
        | Builtin::ArrayExtend
        | Builtin::ArrayPop { .. }
        | Builtin::ArrayRemove
        | Builtin::ArraySwapRemove
        | Builtin::ArrayDrain
        | Builtin::ArrayClear
        | Builtin::ArrayTruncate
        | Builtin::ArrayReserve
        | Builtin::ArrayReserveHint { .. }
        | Builtin::ArraySort
        | Builtin::ArrayReverse
        | Builtin::ArraySwap
        | Builtin::StringPush
        | Builtin::StringPushChar
        | Builtin::StringInsert
        | Builtin::StringRemove
        | Builtin::StringTruncate
        | Builtin::ArenaReset => true,
        Builtin::MemReplace { elem } | Builtin::MemTake { elem, .. } | Builtin::MemSwap { elem } => reshaped_by_write(types, *elem, 0),
        Builtin::SoA { op, .. } => matches!(
            op,
            SoAOp::Push | SoAOp::TryPush | SoAOp::TakeLast | SoAOp::Clear | SoAOp::Reserve | SoAOp::SwapRemove | SoAOp::Swap | SoAOp::Truncate | SoAOp::MoveSlot | SoAOp::DropAt | SoAOp::SetLen
        ),
        _ => false,
    }
}

/// `[EFF-24]` — a built-in that only adds to the end of the list or string its first argument points
/// at, or makes room there.
fn adding_builtin(which: &Builtin) -> bool {
    matches!(
        which,
        Builtin::ArrayPush
            | Builtin::ArrayExtend
            | Builtin::ArrayReserve
            | Builtin::ArrayReserveHint { .. }
            | Builtin::StringPush
            | Builtin::StringPushChar
            | Builtin::SoA { op: SoAOp::Push | SoAOp::TryPush | SoAOp::Reserve, .. }
    )
}

/// Whether a value of this type points at memory the function may reach through it: a `StrView` and a
/// C string at their text, too.
fn points(types: &TypeTable, ty: ember_types::Ty) -> bool {
    matches!(
        types.kind(ty),
        TyKind::Ref { .. } | TyKind::Ptr { .. } | TyKind::Class(_) | TyKind::Span { .. } | TyKind::Vec { .. } | TyKind::Dyn { .. } | TyKind::ClassInterface(_) | TyKind::Str | TyKind::CStr
    )
}

/// A built-in whose result is memory it has just made, which nothing the function had reaches.
fn fresh_builtin(which: &Builtin) -> bool {
    matches!(which, Builtin::ArrayNew | Builtin::StringNew | Builtin::Input)
}

/// What each local holding a pointer may point at, over the whole body (flow-insensitive: every
/// value it is ever given).
fn pointers_of(body: &Body, types: &TypeTable) -> HashMap<LocalId, HashSet<Target>> {
    let mut map: HashMap<LocalId, HashSet<Target>> = HashMap::new();
    let pointer = |local: LocalId| (local.0 as usize) < body.locals.len() && points(types, body.locals[local.0 as usize].ty);
    for block in &body.blocks {
        for stmt in &block.stmts {
            let StmtKind::Assign { place, rvalue } = &stmt.kind else { continue };
            if !place.projection.is_empty() || !pointer(place.local) {
                continue;
            }
            let into = map.entry(place.local).or_default();
            match rvalue {
                Rvalue::Ref { place: lent, .. } => {
                    into.insert(Target::At(Spot { local: lent.local, steps: steps_of(body, types, lent) }));
                }
                Rvalue::Use(Operand::Copy(from) | Operand::Move(from)) | Rvalue::Cast { operand: Operand::Copy(from) | Operand::Move(from), .. } => {
                    let mut steps = steps_of(body, types, from);
                    steps.push(Step::Deref);
                    into.insert(Target::At(Spot { local: from.local, steps }));
                }
                Rvalue::Use(Operand::Const(_)) | Rvalue::Cast { operand: Operand::Const(_), .. } => {}
                _ => {
                    into.insert(Target::Anything);
                }
            }
        }
        // A pointer a call returns points at what its pointer arguments do, or at anything; one to a
        // `static` at the static.
        if let Terminator::Call { func, args, dest, .. } = &block.terminator
            && dest.projection.is_empty()
            && pointer(dest.local)
        {
            let into = map.entry(dest.local).or_default();
            if let FuncRef::Builtin { which: Builtin::LazyStatic { name, ty, .. } | Builtin::ConstStatic { name, ty } | Builtin::ForeignStatic { symbol: name, ty, .. }, .. } = func {
                into.insert(Target::Static(name.to_string(), *ty));
                continue;
            }
            if let FuncRef::Builtin { which, .. } = func
                && fresh_builtin(which)
            {
                continue;
            }
            let mut any = false;
            for operand in args {
                if let Operand::Copy(from) | Operand::Move(from) = operand
                    && points(types, place_type(body, types, from))
                {
                    let mut steps = steps_of(body, types, from);
                    steps.push(Step::Deref);
                    into.insert(Target::At(Spot { local: from.local, steps }));
                    any = true;
                }
            }
            if !any {
                into.insert(Target::Anything);
            }
        }
    }
    map
}

/// The parameter places a spot of the body reaches: a parameter's through a pointer, or through a
/// local pointing at one.
fn reaches(body: &Body, pointers: &HashMap<LocalId, HashSet<Target>>, spot: &Spot, depth: usize, out: &mut Vec<Reach>) {
    let index = spot.local.0 as usize;
    if (1..=body.arg_count).contains(&index) {
        if spot.steps.contains(&Step::Deref) {
            let mut path = spot.steps.clone();
            path.truncate(MAX_PATH);
            out.push(Reach { root: Root::Param(index - 1), path });
        }
        return;
    }
    // Through a local's pointer: `(*p).rest` is the place `p` points at, then `rest`.
    let Some(Step::Deref) = spot.steps.first() else { return };
    if depth > 16 {
        anything(body, out);
        return;
    }
    let Some(held) = pointers.get(&spot.local) else {
        anything(body, out);
        return;
    };
    for target in held {
        match target {
            Target::At(at) => {
                let mut steps = at.steps.clone();
                steps.extend_from_slice(&spot.steps[1..]);
                reaches(body, pointers, &Spot { local: at.local, steps }, depth + 1, out);
            }
            // The static's own place, then the steps past the pointer.
            Target::Static(name, ty) => {
                let mut path = spot.steps[1..].to_vec();
                path.truncate(MAX_PATH);
                out.push(Reach { root: Root::Static(name.clone(), *ty), path });
            }
            Target::Anything => anything(body, out),
        }
    }
}

/// Anything any parameter reaches.
fn anything(body: &Body, out: &mut Vec<Reach>) {
    for param in 0..body.arg_count {
        out.push(Reach { root: Root::Param(param), path: vec![Step::Deref] });
    }
}

#[allow(clippy::too_many_arguments)]
fn body_facts(
    at: usize,
    body: &Body,
    bodies: &[Body],
    types: &TypeTable,
    index: &HashMap<&str, usize>,
    targets: &Targets,
    pointers: &HashMap<LocalId, HashSet<Target>>,
    summaries: &[Summary],
) -> Vec<(What, Fact)> {
    let mut found = Vec::new();
    let mut own = |what: What, place: &Place, found: &mut Vec<(What, Fact)>| {
        let mut out = Vec::new();
        reaches(body, pointers, &Spot { local: place.local, steps: steps_of(body, types, place) }, 0, &mut out);
        found.extend(out.into_iter().map(|reach| (what, Fact { reach, chain: Vec::new() })));
    };
    let read = |operand: &Operand, own: &mut dyn FnMut(What, &Place, &mut Vec<(What, Fact)>), found: &mut Vec<(What, Fact)>| {
        if let Operand::Copy(place) | Operand::Move(place) = operand {
            own(What::Read, place, found);
        }
    };
    for block in &body.blocks {
        for stmt in &block.stmts {
            match &stmt.kind {
                StmtKind::Assign { place, rvalue } => {
                    own(What::Change, place, &mut found);
                    if reshaped_by_write(types, place_type(body, types, place), 0) {
                        own(What::Reshape, place, &mut found);
                        own(What::Remove, place, &mut found);
                    }
                    match rvalue {
                        Rvalue::Use(operand) | Rvalue::Cast { operand, .. } | Rvalue::UnaryOp { operand, .. } | Rvalue::Repeat { value: operand, .. } => {
                            read(operand, &mut own, &mut found);
                        }
                        Rvalue::BinaryOp { lhs, rhs, .. } => {
                            read(lhs, &mut own, &mut found);
                            read(rhs, &mut own, &mut found);
                        }
                        Rvalue::Aggregate { operands, .. } => {
                            for operand in operands {
                                read(operand, &mut own, &mut found);
                            }
                        }
                        Rvalue::Discriminant(place) => own(What::Read, place, &mut found),
                        // What is lent is changed or read by whoever uses the view: followed there.
                        Rvalue::Ref { .. } => {}
                    }
                }
                StmtKind::CheckedBinaryOp { dest, overflow, lhs, rhs, .. } => {
                    own(What::Change, dest, &mut found);
                    own(What::Change, overflow, &mut found);
                    read(lhs, &mut own, &mut found);
                    read(rhs, &mut own, &mut found);
                }
                StmtKind::Drop { place, .. } => own(What::End, place, &mut found),
                _ => {}
            }
        }
        let Terminator::Call { func, args, .. } = &block.terminator else { continue };
        let span = block.terminator_span;
        let callees: Vec<usize> = match func {
            FuncRef::Direct { symbol, .. } => match index.get(symbol.as_str()) {
                Some(&callee) => vec![callee],
                // A function with no body here (C, the runtime): whatever its pointer arguments reach.
                None => {
                    for operand in args {
                        if let Operand::Copy(place) | Operand::Move(place) = operand
                            && points(types, place_type(body, types, place))
                        {
                            let mut through = place.clone();
                            through.projection.push(Projection::Deref);
                            own(What::Change, &through, &mut found);
                        }
                    }
                    Vec::new()
                }
            },
            FuncRef::Indirect { operand, ty, .. } => targets.of_call(at, body, operand, *ty, types),
            FuncRef::Virtual { owner, slot, .. } => bodies
                .iter()
                .enumerate()
                .filter(|(_, candidate)| candidate.class_virtual_slot == Some(*slot) && candidate.class_owner.is_some_and(|class| below(types, class, *owner)))
                .map(|(callee, _)| callee)
                .collect(),
            FuncRef::Interface { interfaces, slot, .. } => targets.of_dyn(interfaces, *slot).to_vec(),
            // A builtin changes what its `ref mut` arguments view and reads what its others view; one
            // that reshapes, what its first argument (both, for `mem.swap`) points at.
            FuncRef::Builtin { which, .. } => {
                if reshaping_builtin(types, which) {
                    let reshaped = if matches!(which, Builtin::MemSwap { .. }) { 2 } else { 1 };
                    for operand in args.iter().take(reshaped) {
                        if let Operand::Copy(place) | Operand::Move(place) = operand
                            && points(types, place_type(body, types, place))
                        {
                            let mut through = place.clone();
                            through.projection.push(Projection::Deref);
                            own(What::Reshape, &through, &mut found);
                            own(if adding_builtin(which) { What::Add } else { What::Remove }, &through, &mut found);
                        }
                    }
                }
                for operand in args {
                    if let Operand::Copy(place) | Operand::Move(place) = operand {
                        let ty = place_type(body, types, place);
                        let what = match types.kind(ty) {
                            TyKind::Ref { mutable: true, .. } | TyKind::Ptr { mutable: true, .. } => Some(What::Change),
                            TyKind::Ref { .. } | TyKind::Ptr { .. } | TyKind::Span { .. } | TyKind::Vec { .. } | TyKind::Class(_) | TyKind::Str | TyKind::CStr => Some(What::Read),
                            _ => None,
                        };
                        if let Some(what) = what {
                            let mut through = place.clone();
                            through.projection.push(Projection::Deref);
                            own(what, &through, &mut found);
                        }
                        own(What::Read, place, &mut found);
                    }
                }
                Vec::new()
            }
            _ => Vec::new(),
        };
        for operand in args {
            read(operand, &mut own, &mut found);
        }
        // The callee's facts, mapped onto the arguments: its parameter `i` is argument `i`.
        for callee in callees {
            let summary = &summaries[callee];
            for (what, facts) in [(What::Change, &summary.changes), (What::Read, &summary.reads), (What::Reshape, &summary.reshapes), (What::Add, &summary.adds), (What::Remove, &summary.removes), (What::End, &summary.ends)] {
                for fact in facts {
                    let param = match &fact.reach.root {
                        Root::Param(param) => *param,
                        // A static is the same place here.
                        Root::Static(..) => {
                            let mut chain = vec![(callee, span)];
                            chain.extend_from_slice(&fact.chain);
                            found.push((what, Fact { reach: fact.reach.clone(), chain }));
                            continue;
                        }
                    };
                    let Some(Operand::Copy(place) | Operand::Move(place)) = args.get(param) else { continue };
                    let mut steps = steps_of(body, types, place);
                    steps.extend_from_slice(&fact.reach.path);
                    let mut out = Vec::new();
                    reaches(body, pointers, &Spot { local: place.local, steps }, 0, &mut out);
                    for reach in out {
                        let mut chain = vec![(callee, span)];
                        chain.extend_from_slice(&fact.chain);
                        found.push((what, Fact { reach, chain }));
                    }
                }
            }
        }
    }
    found
}

/// Whether `class` is `owner` or below it.
fn below(types: &TypeTable, class: ember_types::ClassId, owner: ember_types::ClassId) -> bool {
    let mut current = Some(class);
    while let Some(at) = current {
        if at == owner {
            return true;
        }
        current = types.class_def(at).base;
    }
    false
}

/// `ember build --emit mir` — a body's summary, one line for each kind of fact: each place written
/// from the parameter's name and its fields' (`this.count`, `items[_]`), and the first callee of its
/// chain.
pub fn render(summary: &Summary, body: &Body, bodies: &[Body], types: &TypeTable) -> String {
    let place = |reach: &Reach| {
        // A method's object is `this` in its text (`[FN-4]`); a static is its name.
        let (mut text, mut ty) = match &reach.root {
            Root::Param(param) => {
                let local = body.locals.get(param + 1);
                let name = local.and_then(|local| local.name.clone()).map(|name| if name == "self" { "this".to_string() } else { name });
                (name.unwrap_or_else(|| format!("arg{param}")), local.map(|local| local.ty))
            }
            Root::Static(name, ty) => (name.rsplit('.').next().unwrap_or(name).to_string(), Some(*ty)),
        };
        let mut variant = None;
        for step in &reach.path {
            let current = ty.map(|ty| types.kind(ty).clone());
            let (shown, next) = match (step, current) {
                (Step::Deref, Some(TyKind::Ref { inner, .. } | TyKind::Ptr { inner, .. })) => (String::new(), Some(inner)),
                (Step::Deref, kind) => (String::new(), kind.and(ty)),
                (Step::Field(field) | Step::Column(field), Some(TyKind::Struct(id))) => match types.struct_def(id).fields.get(*field) {
                    Some(def) => (format!(".{}", def.name), Some(def.ty)),
                    None => (format!(".{field}"), None),
                },
                (Step::Field(field), Some(TyKind::Class(id))) => match types.class_field_at(id, *field) {
                    Some(def) => (format!(".{}", def.name), Some(def.ty)),
                    None => (format!(".{field}"), None),
                },
                (Step::Field(field), Some(TyKind::Enum(id))) => match variant.and_then(|variant: usize| types.enum_def(id).variants.get(variant)).and_then(|def| def.fields.get(*field)) {
                    Some(def) => (format!(".{}", def.name), Some(def.ty)),
                    None => (format!(".{field}"), None),
                },
                // A list's or a string's own fields (its length, its buffer) are the list.
                (Step::Field(_), Some(TyKind::Vec { .. })) => (String::new(), ty),
                (Step::Field(field) | Step::Column(field), _) => (format!(".{field}"), None),
                (Step::Variant(at), Some(TyKind::Enum(id))) => {
                    variant = Some(*at);
                    let name = types.enum_def(id).variants.get(*at).map_or_else(|| at.to_string(), |def| def.name.to_string());
                    (format!(" as {name}"), ty)
                }
                (Step::Variant(at), _) => (format!(" as {at}"), None),
                (Step::Elements, Some(TyKind::Vec { elem, .. } | TyKind::Array { elem, .. } | TyKind::ArrayN { elem, .. } | TyKind::Span { elem, .. })) => ("[_]".to_string(), Some(elem)),
                (Step::Elements, _) => ("[_]".to_string(), None),
            };
            text.push_str(&shown);
            ty = next;
        }
        text
    };
    let line = |label: &str, facts: &[Fact]| {
        let mut items: Vec<String> = Vec::new();
        for fact in facts {
            let item = match fact.chain.first() {
                Some((callee, _)) => format!("{} (through {})", place(&fact.reach), bodies[*callee].name),
                None => place(&fact.reach),
            };
            if !items.contains(&item) {
                items.push(item);
            }
        }
        if items.is_empty() { String::new() } else { format!("  {label} {}\n", items.join(", ")) }
    };
    // `[EFF-24]` — of the places it adds to, those it only adds to.
    let only: Vec<Fact> = summary.adds.iter().filter(|fact| summary.only_adds(&fact.reach)).cloned().collect();
    format!(
        "{}{}{}{}{}{}",
        line("changes", &summary.changes),
        line("reads", &summary.reads),
        line("reshapes", &summary.reshapes),
        line("adds to", &summary.adds),
        line("only adds to", &only),
        line("ends", &summary.ends)
    )
}
