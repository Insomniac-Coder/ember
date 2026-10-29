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

use ember_mir::{Body, FuncRef, LocalId, Operand, Place, Projection, Rvalue, StmtKind, Terminator};
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

/// A place a parameter (0-based; a method's `this` is 0) reaches.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Reach {
    pub param: usize,
    pub path: Vec<Step>,
}

/// A fact and the chain of calls that makes it: each call's callee (a body's index) and the call's
/// span; empty for the function's own line.
#[derive(Clone, Debug)]
pub struct Fact {
    pub reach: Reach,
    pub chain: Vec<(usize, Span)>,
}

/// `[EFF-24]` — what a function may change, read and end.
#[derive(Clone, Debug, Default)]
pub struct Summary {
    pub changes: Vec<Fact>,
    pub reads: Vec<Fact>,
    pub ends: Vec<Fact>,
}

impl Summary {
    /// Whether it may change the place `param`'s `path` reaches, or anything inside it or holding it.
    pub fn may_change(&self, param: usize, path: &[Step]) -> bool {
        self.changes.iter().any(|fact| fact.reach.param == param && overlaps(&fact.reach.path, path))
    }

    /// Whether it may change anything parameter `param` reaches: for a method's `this`, whether it
    /// changes its object.
    pub fn changes_param(&self, param: usize) -> bool {
        self.changes.iter().any(|fact| fact.reach.param == param)
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
        for list in [&mut summary.changes, &mut summary.reads, &mut summary.ends] {
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

/// Whether a value of this type points at memory the function may reach through it.
fn points(types: &TypeTable, ty: ember_types::Ty) -> bool {
    matches!(types.kind(ty), TyKind::Ref { .. } | TyKind::Ptr { .. } | TyKind::Class(_) | TyKind::Span { .. } | TyKind::Vec { .. } | TyKind::Dyn { .. } | TyKind::ClassInterface(_))
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
        // A pointer a call returns points at what its pointer arguments do, or at anything.
        if let Terminator::Call { args, dest, .. } = &block.terminator
            && dest.projection.is_empty()
            && pointer(dest.local)
        {
            let into = map.entry(dest.local).or_default();
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
            out.push(Reach { param: index - 1, path });
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
            Target::Anything => anything(body, out),
        }
    }
}

/// Anything any parameter reaches.
fn anything(body: &Body, out: &mut Vec<Reach>) {
    for param in 0..body.arg_count {
        out.push(Reach { param, path: vec![Step::Deref] });
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
            // A builtin changes what its `ref mut` arguments view and reads what its others view.
            FuncRef::Builtin { .. } => {
                for operand in args {
                    if let Operand::Copy(place) | Operand::Move(place) = operand {
                        let ty = place_type(body, types, place);
                        let what = match types.kind(ty) {
                            TyKind::Ref { mutable: true, .. } | TyKind::Ptr { mutable: true, .. } => Some(What::Change),
                            TyKind::Ref { .. } | TyKind::Ptr { .. } | TyKind::Span { .. } | TyKind::Vec { .. } | TyKind::Class(_) => Some(What::Read),
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
            for (what, facts) in [(What::Change, &summary.changes), (What::Read, &summary.reads), (What::End, &summary.ends)] {
                for fact in facts {
                    let Some(Operand::Copy(place) | Operand::Move(place)) = args.get(fact.reach.param) else { continue };
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
/// from the parameter's name and its fields' (`self.count`, `items[_]`), and the first callee of its
/// chain.
pub fn render(summary: &Summary, body: &Body, bodies: &[Body], types: &TypeTable) -> String {
    let place = |reach: &Reach| {
        let local = body.locals.get(reach.param + 1);
        let mut text = local.and_then(|local| local.name.clone()).unwrap_or_else(|| format!("arg{}", reach.param));
        let mut ty = local.map(|local| local.ty);
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
    format!("{}{}{}", line("changes", &summary.changes), line("reads", &summary.reads), line("ends", &summary.ends))
}
