//! `[TYP-15]` — where a view may be stored (D-352, D-353, D-198; ODR-069).
//!
//! A view may be stored only in a place bounded by every region it carries.
//! [`Regions`] follows each store to where it lands; this module judges what
//! it finds. A store into storage no local bounds (an `Array`'s element, a
//! class object, a `Box`, a span's element, the target of a reference the
//! analysis cannot follow) needs a `static` view, and a view stored into the
//! place a `mut` argument points to reaches the caller's place.
//!
//! Where the stored view came from a parameter, the function's callers answer
//! for it: its [`StoreSummary`] says which argument slots must be `static` and
//! which flow into which argument's place, and each caller checks its own
//! arguments against it. So generic code such as `Map.insert` works for a
//! `str` key its caller passes as a literal, and refuses one borrowed from a
//! local `String`. A function that can be called where no summary is read (a
//! virtual method, a closure, a function value, a `dyn` adapter) answers for
//! it itself.

use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};

use ember_diag::{Diagnostic, Sink, codes};
use ember_mir::{Body, CastKind, Const, FuncRef, LocalId, Operand, Projection, Rvalue, StmtKind, Terminator};
use ember_span::Span;
use ember_types::{Ty, TyKind, TypeTable, UintTy};

use crate::regions::{
    ArgSlot, CallRegionContract, CallerPlaceFact, Origin, Point, RegionVid, Regions, StoreRequirement,
    StoreSummary, Unbounded, place_type,
};

/// The bodies that can be called where no store summary is read: virtual
/// methods, closures, functions used as values, `dyn` adapters' methods and
/// functions with a foreign ABI.
pub fn dynamic_bodies(bodies: &[Body]) -> HashSet<String> {
    let mut dynamic: HashSet<String> = bodies
        .iter()
        .filter(|body| body.class_virtual_slot.is_some() || body.is_lambda || body.abi.is_some())
        .map(|body| body.symbol.clone())
        .collect();
    let note_operand = |operand: &Operand, dynamic: &mut HashSet<String>| {
        if let Operand::Const(Const::Fn(symbol)) = operand {
            dynamic.insert(symbol.clone());
        }
    };
    for body in bodies {
        for block in &body.blocks {
            for stmt in &block.stmts {
                let StmtKind::Assign { rvalue, .. } = &stmt.kind else { continue };
                match rvalue {
                    Rvalue::Use(operand) | Rvalue::UnaryOp { operand, .. } | Rvalue::Repeat { value: operand, .. } => {
                        note_operand(operand, &mut dynamic)
                    }
                    Rvalue::Cast { operand, kind, .. } => {
                        note_operand(operand, &mut dynamic);
                        if let CastKind::InterfaceUpcast { implementations, .. }
                        | CastKind::ClassInterfaceUpcast { implementations, .. } = kind
                        {
                            dynamic.extend(implementations.iter().flatten().map(|method| method.symbol.clone()));
                        }
                    }
                    Rvalue::BinaryOp { lhs, rhs, .. } => {
                        note_operand(lhs, &mut dynamic);
                        note_operand(rhs, &mut dynamic);
                    }
                    Rvalue::Aggregate { operands, .. } => {
                        for operand in operands {
                            note_operand(operand, &mut dynamic);
                        }
                    }
                    Rvalue::Ref { .. } | Rvalue::Discriminant(_) => {}
                }
            }
            let Terminator::Call { func, args, .. } = &block.terminator else { continue };
            for operand in args {
                note_operand(operand, &mut dynamic);
            }
            match func {
                FuncRef::Indirect { operand, .. } => note_operand(operand, &mut dynamic),
                FuncRef::DynBoxNew { implementations, .. } => {
                    dynamic.extend(implementations.iter().flatten().map(|method| method.symbol.clone()));
                }
                _ => {}
            }
        }
    }
    dynamic
}

/// `contract` with the store summary of its direct callee, if it has one.
pub fn with_stores(
    mut contract: CallRegionContract,
    func: &FuncRef,
    summaries: &HashMap<String, StoreSummary>,
) -> CallRegionContract {
    if let FuncRef::Direct { symbol, .. } = func
        && let Some(summary) = summaries.get(symbol.as_str())
    {
        contract.stores = summary.clone();
    }
    contract
}

/// Infer every body's store summary to a fixpoint: a caller's depends on its
/// callees', so a body is summarised again whenever a callee's grows.
pub fn infer_store_summaries(
    bodies: &[Body],
    types: &TypeTable,
    base: &dyn Fn(&FuncRef) -> CallRegionContract,
    capture_paths: &dyn Fn(&Body) -> HashMap<Point, Vec<Vec<Projection>>>,
    dynamic: &HashSet<String>,
) -> HashMap<String, StoreSummary> {
    let index: HashMap<&str, usize> =
        bodies.iter().enumerate().map(|(position, body)| (body.symbol.as_str(), position)).collect();
    let mut callers: Vec<Vec<usize>> = vec![Vec::new(); bodies.len()];
    for (position, body) in bodies.iter().enumerate() {
        for block in &body.blocks {
            if let Terminator::Call { func: FuncRef::Direct { symbol, .. }, .. } = &block.terminator
                && let Some(&callee) = index.get(symbol.as_str())
                && !callers[callee].contains(&position)
            {
                callers[callee].push(position);
            }
        }
    }
    let eligible = |body: &Body| {
        !dynamic.contains(&body.symbol) && body.args().any(|(_, decl)| types.is_view(decl.ty))
    };
    let mut summaries: HashMap<String, StoreSummary> = HashMap::new();
    let mut queue: VecDeque<usize> = (0..bodies.len()).filter(|&position| eligible(&bodies[position])).collect();
    let mut queued: HashSet<usize> = queue.iter().copied().collect();
    let mut rounds = 0usize;
    let limit = bodies.len().saturating_mul(8).max(64);
    while let Some(position) = queue.pop_front() {
        queued.remove(&position);
        rounds += 1;
        if rounds > limit {
            break;
        }
        let body = &bodies[position];
        let contract = |func: &FuncRef| with_stores(base(func), func, &summaries);
        let regions = Regions::infer_with_capture_borrow_paths(body, types, &contract, &capture_paths(body));
        let summary = summarize(body, types, &regions, &contract);
        let previous = summaries.get(&body.symbol).cloned().unwrap_or_default();
        if summary == previous {
            continue;
        }
        if summary.static_slots.is_empty() && summary.flows.is_empty() {
            summaries.remove(&body.symbol);
        } else {
            summaries.insert(body.symbol.clone(), summary);
        }
        for &caller in &callers[position] {
            if eligible(&bodies[caller]) && queued.insert(caller) {
                queue.push_back(caller);
            }
        }
    }
    summaries
}

fn summarize(
    body: &Body,
    types: &TypeTable,
    regions: &Regions,
    call_contract: &dyn Fn(&FuncRef) -> CallRegionContract,
) -> StoreSummary {
    let mut summary = StoreSummary::default();
    for requirement in regions.store_requirements(body, types, call_contract) {
        let judgement = judge(body, types, regions, requirement.ty, &requirement.roots, &requirement.origins, None);
        summary.static_slots.extend(judgement.callers);
    }
    for fact in regions.caller_place_facts(body, types) {
        let ty = slot_type(body, types, fact.arg, &fact.projection);
        let judgement = judge(body, types, regions, ty, &fact.roots, &fact.origins, Some((fact.arg, fact.slot)));
        let into = (fact.arg.0 as usize - 1, fact.projection.clone());
        for from in judgement.callers {
            summary.flows.insert((from, into.clone()));
        }
    }
    summary
}

/// What a stored view carries, judged: the parameter slots it came from
/// (the callers', to supply), and the first thing that makes it an error.
struct Judgement {
    callers: BTreeSet<ArgSlot>,
    culprit: Option<Culprit>,
}

#[derive(Clone, Copy)]
enum Culprit {
    /// A borrow of, or a view into, this local (or parameter's own storage).
    Local(LocalId),
    /// A region that ends with a callback's call (`@latebound`).
    LateBound,
}

/// `own`, for a caller's place, is that place's parameter and slot: what it
/// already held is not a new store.
fn judge(
    body: &Body,
    types: &TypeTable,
    regions: &Regions,
    ty: Ty,
    roots: &HashSet<RegionVid>,
    origins: &HashSet<Origin>,
    own: Option<(LocalId, RegionVid)>,
) -> Judgement {
    let mut callers = BTreeSet::new();
    let mut culprit = None;
    let mut parameters = HashSet::new();
    for &root in roots {
        if let Some(place) = regions.loan_place(root) {
            if !copies_only(types, place_type(body, types, place), ty) {
                culprit.get_or_insert(Culprit::Local(place.local));
            }
        } else if let Some((parameter, path)) = regions.entry_slot(body, root) {
            parameters.insert(parameter);
            if own.is_some_and(|(arg, slot)| arg == parameter && slot == root)
                || parameter_copies_only(types, body.local(parameter).ty, ty)
            {
                continue;
            }
            callers.insert((parameter.0 as usize - 1, path));
        }
    }
    for origin in origins {
        match *origin {
            Origin::Local(local) => {
                if !copies_only(types, body.local(local).ty, ty) {
                    culprit.get_or_insert(Culprit::Local(local));
                }
            }
            Origin::Param(local) => {
                if parameters.contains(&local)
                    || own.is_some_and(|(arg, _)| arg == local)
                    || parameter_copies_only(types, body.local(local).ty, ty)
                {
                    continue;
                }
                culprit.get_or_insert(Culprit::Local(local));
            }
            Origin::LateBound { .. } => {
                culprit.get_or_insert(Culprit::LateBound);
            }
        }
    }
    Judgement { callers, culprit }
}

/// What a view of type `value` may point into: text's bytes, or a span's
/// elements. A reference or a view struct may point anywhere.
#[derive(Clone, Copy)]
enum Viewed {
    Bytes,
    Elements(Ty),
}

fn viewed(types: &TypeTable, value: Ty) -> Option<Viewed> {
    match *types.kind(value) {
        TyKind::Str => Some(Viewed::Bytes),
        TyKind::Span { elem, .. } => Some(Viewed::Elements(elem)),
        _ => None,
    }
}

/// Whether a view of type `value` got by borrowing a `container` can only be
/// a copy of a view kept in it: the container owns no storage such a view
/// could point into (a `str` into a `String`'s bytes, a `Span[T]` into an
/// `Array[T]`). Such a view's own regions are carried with it (a local's
/// slots) or `static` (a heap element's), so the borrow adds nothing.
fn copies_only(types: &TypeTable, container: Ty, value: Ty) -> bool {
    let Some(target) = viewed(types, value) else { return false };
    !owns_viewable(types, container, target, &mut HashSet::new())
}

/// As [`copies_only`], for a parameter: a view parameter's own regions are
/// its caller's, not a container's, so only a reference to a container
/// with no views of its own qualifies (`names: Array[str]`, passed by
/// address).
fn parameter_copies_only(types: &TypeTable, parameter: Ty, value: Ty) -> bool {
    match *types.kind(parameter) {
        TyKind::Ref { inner, .. } => !types.is_view(inner) && copies_only(types, inner, value),
        _ => false,
    }
}

fn owns_viewable(types: &TypeTable, ty: Ty, target: Viewed, seen: &mut HashSet<Ty>) -> bool {
    if !seen.insert(ty) {
        return false;
    }
    let holds = |elem: Ty| match target {
        Viewed::Bytes => matches!(types.kind(elem), TyKind::Uint(UintTy::U8)),
        Viewed::Elements(wanted) => elem == wanted,
    };
    match types.kind(ty) {
        TyKind::Vec { elem, .. } | TyKind::Array { elem, .. } | TyKind::Span { elem, .. } => {
            holds(*elem) || owns_viewable(types, *elem, target, seen)
        }
        // Through a reference, what it points to: a `str` got from a
        // `ref String` points into the `String`.
        TyKind::Ref { inner, .. } => owns_viewable(types, *inner, target, seen),
        TyKind::Struct(id) => {
            let fields: Vec<Ty> = types.struct_def(*id).fields.iter().map(|field| field.ty).collect();
            fields.into_iter().any(|field| owns_viewable(types, field, target, seen))
        }
        TyKind::Tuple(items) => {
            let items = items.clone();
            items.into_iter().any(|item| owns_viewable(types, item, target, seen))
        }
        TyKind::Enum(id) => {
            let fields: Vec<Ty> = types
                .enum_def(*id)
                .variants
                .iter()
                .flat_map(|variant| variant.fields.iter().map(|field| field.ty))
                .collect();
            fields.into_iter().any(|field| owns_viewable(types, field, target, seen))
        }
        TyKind::Class(id) => {
            let count = types.class_field_count(*id);
            let fields: Vec<Ty> = (0..count).filter_map(|index| types.class_field_at(*id, index).map(|field| field.ty)).collect();
            fields.into_iter().any(|field| owns_viewable(types, field, target, seen))
        }
        _ => false,
    }
}

fn slot_type(body: &Body, types: &TypeTable, arg: LocalId, path: &[Projection]) -> Ty {
    let mut ty = body.local(arg).ty;
    for projection in path {
        ty = match (projection, types.kind(ty)) {
            (Projection::Deref, TyKind::Ref { inner, .. }) => *inner,
            (Projection::Field(i), TyKind::Struct(id)) => types.struct_def(*id).fields.get(*i).map_or(ty, |f| f.ty),
            (Projection::Field(i), TyKind::Tuple(items)) => items.get(*i).copied().unwrap_or(ty),
            (Projection::Index(_) | Projection::ConstIndex(_), TyKind::Array { elem, .. }) => *elem,
            _ => ty,
        };
    }
    ty
}

/// `[TYP-15]` — report each store the body cannot answer for. `dynamic`: the
/// body can be called where no summary is read, so a parameter's view is
/// not its callers' to supply.
pub fn check_stores(
    body: &Body,
    types: &TypeTable,
    regions: &Regions,
    call_contract: &dyn Fn(&FuncRef) -> CallRegionContract,
    dynamic: bool,
    names: &HashMap<String, String>,
    sink: &mut Sink,
) {
    let mut reported = HashSet::new();
    for requirement in regions.store_requirements(body, types, call_contract) {
        let judgement = judge(body, types, regions, requirement.ty, &requirement.roots, &requirement.origins, None);
        let callers = !judgement.callers.is_empty();
        if judgement.culprit.is_none() && !(dynamic && callers) {
            continue;
        }
        if !reported.insert((requirement.point, requirement.call.clone())) {
            continue;
        }
        let span = point_span(body, requirement.point);
        sink.emit_classified(requirement_error(body, types, &requirement, judgement.culprit, dynamic, names, span));
    }
    for fact in regions.caller_place_facts(body, types) {
        let ty = slot_type(body, types, fact.arg, &fact.projection);
        let judgement = judge(body, types, regions, ty, &fact.roots, &fact.origins, Some((fact.arg, fact.slot)));
        let callers = !judgement.callers.is_empty();
        if judgement.culprit.is_none() && !(dynamic && callers) {
            continue;
        }
        if !reported.insert((fact.point, Some((String::new(), fact.arg.0 as usize)))) {
            continue;
        }
        let span = regions
            .first_store_into(body, fact.slot, &fact.roots, &fact.origins)
            .unwrap_or_else(|| point_span(body, fact.point));
        sink.emit_classified(caller_place_error(body, types, &fact, ty, judgement.culprit, span));
    }
}

/// How a diagnostic names the body: a closure has no name of its own.
fn callable_name(body: &Body) -> String {
    if body.is_lambda { String::from("a closure") } else { format!("`{}`", body.name) }
}

fn point_span(body: &Body, point: Point) -> Span {
    let block = &body.blocks[point.block];
    match block.stmts.get(point.index) {
        Some(stmt) => stmt.span,
        None => block.terminator_span,
    }
}

fn local_name(body: &Body, local: LocalId) -> Option<String> {
    body.local(local).name.clone().filter(|name| !name.is_empty())
}

const OWNED_COPY_HELP: &str = "store an owned copy — `String` for `str`, `Array[T]` for `Span[T]` — and note \
     that costs one allocation per element; or store a `u32` index or a `Handle[T]` and name the \
     container it indexes";

fn requirement_error(
    body: &Body,
    types: &TypeTable,
    requirement: &StoreRequirement,
    culprit: Option<Culprit>,
    dynamic: bool,
    names: &HashMap<String, String>,
    span: Span,
) -> Diagnostic {
    let shown = types.display(requirement.ty);
    let (message, label) = match &requirement.call {
        Some((callee, _)) => {
            let callee = names.get(callee).cloned().unwrap_or_else(|| callee.clone());
            (
                format!("`{shown}` is a view, so it may not be passed to `{callee}`, which stores it where only a `static` view may go"),
                "stored by this call",
            )
        }
        None => (
            match requirement.target {
                Unbounded::Element => format!("`{shown}` is a view, so it may not be stored in an `Array` unless it is `static`"),
                Unbounded::ClassField => format!("`{shown}` is a view, so it may not be stored in a class object unless it is `static`"),
                Unbounded::SpanElement => format!("`{shown}` is a view, so it may not be stored in a span's element unless it is `static`"),
                Unbounded::BoxContents => format!("`{shown}` is a view, so it may not be stored in a Box's contents"),
                Unbounded::SharedContents => format!("`{shown}` is a view, so it may not be stored in a Shared's contents"),
                Unbounded::Unknown => format!(
                    "`{shown}` is a view, so it may not be stored through a reference whose target cannot be seen, unless it is `static`"
                ),
            },
            "stored here",
        ),
    };
    let mut diagnostic = Diagnostic::error(codes::E3063, span, message).primary_label(label).help(OWNED_COPY_HELP);
    diagnostic = match culprit {
        Some(Culprit::Local(local)) => match local_name(body, local) {
            Some(name) => diagnostic.note(format!(
                "the view borrows `{name}`, and this place has no bounding region: only a view with the `static` region, such as a string literal, may be stored in it (TYP-15, LT-3)"
            )),
            None => diagnostic.note(
                "this place has no bounding region, so only a view with the `static` region, such as a string literal, may be stored in it (TYP-15, LT-3)",
            ),
        },
        Some(Culprit::LateBound) => diagnostic.note(
            "the view is valid only during a callback's call; this place has no bounding region, so only a `static` view may be stored in it (TYP-15, LT-7)",
        ),
        None if dynamic => diagnostic.note(format!(
            "the view is a parameter's, and {} can be called where its callers are not checked (a virtual method, a closure or a function value), so it may store only `static` views here (TYP-15)",
            callable_name(body)
        )),
        None => diagnostic,
    };
    diagnostic
}

fn caller_place_error(
    body: &Body,
    types: &TypeTable,
    fact: &CallerPlaceFact,
    ty: Ty,
    culprit: Option<Culprit>,
    span: Span,
) -> Diagnostic {
    let shown = types.display(ty);
    // A closure's environment is its captured variables, its creator's.
    let (parameter, owner) = if body.is_lambda && body.closure_environment.is_some() && fact.arg == LocalId(1) {
        (String::from("a captured variable"), "the closure's creator")
    } else {
        (format!("`{}`", local_name(body, fact.arg).unwrap_or_else(|| String::from("the parameter"))), "the caller")
    };
    let (message, note) = match culprit {
        Some(Culprit::Local(local)) => {
            let what = local_name(body, local).unwrap_or_else(|| String::from("a local"));
            (
                format!("a view of `{what}` is stored in {parameter}, which {owner} owns, so it would outlive `{what}`"),
                format!(
                    "a `mut` parameter is the caller's place: a `{shown}` stored in it must outlive the call, and `{what}` ends with it (TYP-15, FN-1)"
                ),
            )
        }
        Some(Culprit::LateBound) => (
            format!("a callback's view is stored in {parameter}, which {owner} owns"),
            String::from("the view is valid only during the callback's call (TYP-15, LT-7)"),
        ),
        None => (
            format!("a parameter's view is stored in {parameter}, which {owner} owns"),
            format!(
                "{} can be called where its callers are not checked (a virtual method, a closure or a function value), so it may store only `static` views, or views of {parameter} itself, in {parameter} (TYP-15)",
                callable_name(body)
            ),
        ),
    };
    Diagnostic::error(codes::E3063, span, message)
        .primary_label("stored here")
        .help("return the view instead, or store an owned copy (`String` for `str`)")
        .note(note)
}
