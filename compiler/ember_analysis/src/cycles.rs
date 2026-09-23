//! Static strong-ownership cycle diagnostics (`[WK-5]`–`[WK-7]`).
//!
//! This is a declaration-time lint, not a collector or a rejection rule. The
//! graph follows every statically known owning path from a class field to a
//! class handle and deliberately stops at `Weak[...]`, references, pointers,
//! and views. A reported cycle is therefore possible rather than certain at
//! run time, which is precisely the conservative contract of `[WK-7]`.

use std::collections::{BTreeSet, HashSet, VecDeque};

use ember_diag::{Diagnostic, Sink, codes};
use ember_span::Span;
use ember_types::{ClassId, Ty, TyKind, TypeTable};

#[derive(Clone)]
struct StrongEdge {
    from: usize,
    to: usize,
    field_owner: usize,
    field: String,
    field_type: String,
    type_span: Span,
    weak_replacement: Option<String>,
    span: Span,
    order: usize,
}

/// The ownership classification a user-visible graph edge carries.
///
/// `Unknown` is deliberately distinct from both `Strong` and `Weak`: an
/// unresolved generic or opaque dynamic owner is not evidence that an edge is
/// non-owning, but it cannot participate in a statically proven SCC either.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OwnershipEdgeKind {
    Strong,
    Weak,
    Unknown,
}

impl OwnershipEdgeKind {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Strong => "strong",
            Self::Weak => "weak",
            Self::Unknown => "unknown",
        }
    }
}

/// One edge in the source-level ownership graph exposed by `[CLI-17]`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OwnershipEdge {
    pub source: String,
    pub declaration: String,
    pub field: String,
    pub field_type: String,
    pub target: Option<String>,
    pub kind: OwnershipEdgeKind,
}

/// One shortest all-strong cycle for a strongly connected component.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OwnershipCycle {
    pub edges: Vec<OwnershipEdge>,
}

/// The static data that `ember inspect --cycle` presents. The same strong
/// graph is used by the declaration-time `L3001` lint; weak and unknown edges
/// are retained here so a report never silently calls them strong.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OwnershipInspection {
    pub edges: Vec<OwnershipEdge>,
    pub shortest_cycles: Vec<OwnershipCycle>,
}

/// `[WK-5]` / `[WK-6]` — warn once for the shortest visible strong cycle in
/// each strongly connected component. The warning is emitted after type
/// collection, so generic instances are examined through their substituted
/// semantic types rather than source spelling.
pub fn lint_strong_cycles(types: &TypeTable, sink: &mut Sink) {
    let graph = ownership_graph(types);
    for component in strongly_connected_components(&graph) {
        let Some(cycle) = shortest_cycle(&graph, &component) else {
            continue;
        };
        report_cycle(types, &cycle, sink);
    }
}

/// `[CLI-17]` — expose the package-visible ownership graph without changing
/// program acceptance or ownership semantics. Each strong SCC contributes its
/// shortest visible cycle, just as it does for `L3001`.
pub fn inspect_ownership_graph(types: &TypeTable) -> OwnershipInspection {
    let graph = ownership_graph(types);
    let mut edges: Vec<_> = graph
        .iter()
        .flatten()
        .map(|edge| inspect_strong_edge(types, edge))
        .collect();

    for (source, _) in types.classes() {
        for field_index in 0..types.class_field_count(source) {
            let (field_owner, field) = types
                .class_field_at_info(source, field_index)
                .expect("class field count must resolve every inherited field");
            let source = class_name(types, source);
            let declaration = class_name(types, field_owner);
            let field_type = ownership_type_name(types, field.ty);
            for target in weak_targets(types, field.ty) {
                edges.push(OwnershipEdge {
                    source: source.clone(),
                    declaration: declaration.clone(),
                    field: field.name.to_string(),
                    field_type: field_type.clone(),
                    target: Some(class_name(types, target)),
                    kind: OwnershipEdgeKind::Weak,
                });
            }
            if contains_unknown_owner(types, field.ty) {
                edges.push(OwnershipEdge {
                    source,
                    declaration,
                    field: field.name.to_string(),
                    field_type,
                    target: None,
                    kind: OwnershipEdgeKind::Unknown,
                });
            }
        }
    }

    let shortest_cycles = strongly_connected_components(&graph)
        .into_iter()
        .filter_map(|component| shortest_cycle(&graph, &component))
        .map(|cycle| OwnershipCycle {
            edges: cycle
                .iter()
                .map(|edge| inspect_strong_edge(types, edge))
                .collect(),
        })
        .collect();
    OwnershipInspection { edges, shortest_cycles }
}

fn ownership_graph(types: &TypeTable) -> Vec<Vec<StrongEdge>> {
    let mut graph = vec![Vec::new(); types.classes().count()];
    let mut order = 0;

    for (source, _) in types.classes() {
        for field_index in 0..types.class_field_count(source) {
            let (field_owner, field) = types
                .class_field_at_info(source, field_index)
                .expect("class field count must resolve every inherited field");
            for target in strong_targets(types, field.ty) {
                graph[source.0 as usize].push(StrongEdge {
                    from: source.0 as usize,
                    to: target.0 as usize,
                    field_owner: field_owner.0 as usize,
                    field: field.name.to_string(),
                    field_type: ownership_type_name(types, field.ty),
                    type_span: field.ty_span,
                    weak_replacement: direct_weak_replacement(types, field_owner, field.ty),
                    span: field.span,
                    order,
                });
                order += 1;
            }
        }
    }
    graph
}

/// A direct class handle has an unambiguous back-reference replacement. The
/// replacement is intentionally withheld for `Shared`, containers, and fields
/// originating in a generic class recipe: changing those types can alter a
/// declared ownership contract, which `[WK-10]` forbids an automatic edit from
/// doing.
fn direct_weak_replacement(types: &TypeTable, field_owner: ClassId, ty: Ty) -> Option<String> {
    if types.class_def(field_owner).origin.is_some() {
        return None;
    }
    let TyKind::Class(target) = types.kind(ty) else {
        return None;
    };
    Some(format!("Weak[{}]", class_name(types, *target)))
}

fn inspect_strong_edge(types: &TypeTable, edge: &StrongEdge) -> OwnershipEdge {
    OwnershipEdge {
        source: class_name(types, ClassId(edge.from as u32)),
        declaration: class_name(types, ClassId(edge.field_owner as u32)),
        field: edge.field.clone(),
        field_type: edge.field_type.clone(),
        target: Some(class_name(types, ClassId(edge.to as u32))),
        kind: OwnershipEdgeKind::Strong,
    }
}

/// Type-table identity names generic instances with an internal separator so
/// separate instances are unambiguous to the compiler. Cycle reports are a
/// source-facing CLI, however, and `[WK-9]` requires an instantiated ownership
/// type in Ember spelling rather than that compiler-private identity.
fn ownership_type_name(types: &TypeTable, ty: Ty) -> String {
    match types.kind(ty) {
        TyKind::Struct(id) => {
            let def = types.struct_def(*id);
            let Some((name, arguments)) = &def.origin else {
                return types.display(ty);
            };
            let arguments = arguments
                .iter()
                .map(|argument| ownership_type_name(types, *argument))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{name}[{arguments}]")
        }
        TyKind::Class(id) => class_name(types, *id),
        _ => types.display(ty),
    }
}

/// All class handles retained by a value of `ty`. `Weak[O]` is the one
/// compiler-known wrapper that intentionally contributes no edge; `Shared`,
/// `Box`, arrays, tuples, ordinary structs, and payload enums recurse through
/// their owned contents. References and views are non-owning by construction.
fn strong_targets(types: &TypeTable, ty: Ty) -> Vec<ClassId> {
    let mut targets = BTreeSet::new();
    let mut seen = HashSet::new();
    collect_strong_targets(types, ty, &mut seen, &mut targets);
    targets.into_iter().collect()
}

fn collect_strong_targets(
    types: &TypeTable,
    ty: Ty,
    seen: &mut HashSet<Ty>,
    targets: &mut BTreeSet<ClassId>,
) {
    if !seen.insert(ty) {
        return;
    }
    match types.kind(ty) {
        TyKind::Class(class) => {
            targets.insert(*class);
        }
        // An `[OBJ-2]` class-interface handle owns a real class object, but
        // its erased declaration cannot name one statically. Keep it out of
        // the concrete SCC graph; `contains_unknown_owner` records the
        // conservative unknown edge below instead of inventing a target.
        TyKind::ClassInterface(_) => {}
        TyKind::Struct(id) => {
            let def = types.struct_def(*id);
            if def.origin.as_ref().is_some_and(|(name, _)| name.is("Weak")) {
                return;
            }
            if let Some((name, arguments)) = &def.origin
                && (name.is("Shared") || name.is("Box"))
            {
                for argument in arguments {
                    collect_strong_targets(types, *argument, seen, targets);
                }
                return;
            }
            for field in &def.fields {
                collect_strong_targets(types, field.ty, seen, targets);
            }
        }
        TyKind::Enum(id) => {
            for variant in &types.enum_def(*id).variants {
                for field in &variant.fields {
                    collect_strong_targets(types, field.ty, seen, targets);
                }
            }
        }
        TyKind::Tuple(items) => {
            for item in items {
                collect_strong_targets(types, *item, seen, targets);
            }
        }
        TyKind::Array { elem, .. } | TyKind::Vec { elem } => {
            collect_strong_targets(types, *elem, seen, targets);
        }
        TyKind::Bool
        | TyKind::Char
        | TyKind::Int(_)
        | TyKind::Uint(_)
        | TyKind::Float(_)
        | TyKind::Void
        | TyKind::Never
        | TyKind::Str
        | TyKind::Span { .. }
        | TyKind::Range(_)
        | TyKind::Ref { .. }
        | TyKind::Ptr { .. }
        | TyKind::Fn { .. }
        | TyKind::Dyn { .. }
        | TyKind::Param { .. }
        | TyKind::Assoc { .. }
        | TyKind::Infer(_)
        | TyKind::IntLit
        | TyKind::FloatLit
        | TyKind::Error => {}
    }
}

/// Retained weak handles are shown by inspection but never passed to the SCC
/// traversal. A `Weak[Shared[T]]` still identifies `T` as its possible target;
/// its weak outer owner controls the edge classification.
fn weak_targets(types: &TypeTable, ty: Ty) -> Vec<ClassId> {
    let mut targets = BTreeSet::new();
    let mut seen = HashSet::new();
    collect_weak_targets(types, ty, &mut seen, &mut targets);
    targets.into_iter().collect()
}

fn collect_weak_targets(
    types: &TypeTable,
    ty: Ty,
    seen: &mut HashSet<Ty>,
    targets: &mut BTreeSet<ClassId>,
) {
    if !seen.insert(ty) {
        return;
    }
    match types.kind(ty) {
        TyKind::Struct(id) => {
            let def = types.struct_def(*id);
            if let Some((name, arguments)) = &def.origin
                && name.is("Weak")
            {
                for argument in arguments {
                    targets.extend(strong_targets(types, *argument));
                }
                return;
            }
            for field in &def.fields {
                collect_weak_targets(types, field.ty, seen, targets);
            }
        }
        TyKind::Enum(id) => {
            for variant in &types.enum_def(*id).variants {
                for field in &variant.fields {
                    collect_weak_targets(types, field.ty, seen, targets);
                }
            }
        }
        TyKind::Tuple(items) => {
            for item in items {
                collect_weak_targets(types, *item, seen, targets);
            }
        }
        TyKind::Array { elem, .. } | TyKind::Vec { elem } => {
            collect_weak_targets(types, *elem, seen, targets);
        }
        TyKind::Bool
        | TyKind::Char
        | TyKind::Int(_)
        | TyKind::Uint(_)
        | TyKind::Float(_)
        | TyKind::Void
        | TyKind::Never
        | TyKind::Str
        | TyKind::Span { .. }
        | TyKind::Class(_)
        | TyKind::ClassInterface(_)
        | TyKind::Range(_)
        | TyKind::Ref { .. }
        | TyKind::Ptr { .. }
        | TyKind::Fn { .. }
        | TyKind::Dyn { .. }
        | TyKind::Param { .. }
        | TyKind::Assoc { .. }
        | TyKind::Infer(_)
        | TyKind::IntLit
        | TyKind::FloatLit
        | TyKind::Error => {}
    }
}

/// A dynamic object or unresolved type parameter may later retain a class
/// handle. It is an unknown edge for inspection, never an invented strong edge
/// for a warning. Raw pointers, references, and views are explicitly
/// non-owning under `[WK-5]` and so do not appear in this vocabulary.
fn contains_unknown_owner(types: &TypeTable, ty: Ty) -> bool {
    let mut seen = HashSet::new();
    contains_unknown_owner_inner(types, ty, &mut seen)
}

fn contains_unknown_owner_inner(types: &TypeTable, ty: Ty, seen: &mut HashSet<Ty>) -> bool {
    if !seen.insert(ty) {
        return false;
    }
    match types.kind(ty) {
        TyKind::Dyn { .. }
        | TyKind::ClassInterface(_)
        | TyKind::Param { .. }
        | TyKind::Assoc { .. } => true,
        TyKind::Struct(id) => {
            let def = types.struct_def(*id);
            if def.origin.as_ref().is_some_and(|(name, _)| name.is("Weak")) {
                return false;
            }
            def.fields
                .iter()
                .any(|field| contains_unknown_owner_inner(types, field.ty, seen))
        }
        TyKind::Enum(id) => types.enum_def(*id).variants.iter().any(|variant| {
            variant
                .fields
                .iter()
                .any(|field| contains_unknown_owner_inner(types, field.ty, seen))
        }),
        TyKind::Tuple(items) => items
            .iter()
            .any(|item| contains_unknown_owner_inner(types, *item, seen)),
        TyKind::Array { elem, .. } | TyKind::Vec { elem } => {
            contains_unknown_owner_inner(types, *elem, seen)
        }
        TyKind::Bool
        | TyKind::Char
        | TyKind::Int(_)
        | TyKind::Uint(_)
        | TyKind::Float(_)
        | TyKind::Void
        | TyKind::Never
        | TyKind::Str
        | TyKind::Span { .. }
        | TyKind::Class(_)
        | TyKind::Range(_)
        | TyKind::Ref { .. }
        | TyKind::Ptr { .. }
        | TyKind::Fn { .. }
        | TyKind::Infer(_)
        | TyKind::IntLit
        | TyKind::FloatLit
        | TyKind::Error => false,
    }
}

/// A straightforward reachability formulation of SCCs. Class graphs are
/// package-sized and this makes the graph vocabulary and deterministic choice
/// below more legible than a second low-link representation.
fn strongly_connected_components(graph: &[Vec<StrongEdge>]) -> Vec<Vec<usize>> {
    let mut reverse = vec![Vec::new(); graph.len()];
    for edges in graph {
        for edge in edges {
            reverse[edge.to].push(edge.from);
        }
    }

    let mut remaining: BTreeSet<usize> = (0..graph.len()).collect();
    let mut components = Vec::new();
    while let Some(&root) = remaining.first() {
        let forward = reachable(graph.len(), root, |node| {
            graph[node].iter().map(|edge| edge.to)
        });
        let backward = reachable(graph.len(), root, |node| reverse[node].iter().copied());
        let component: Vec<_> = remaining
            .iter()
            .copied()
            .filter(|node| forward[*node] && backward[*node])
            .collect();
        for node in &component {
            remaining.remove(node);
        }
        components.push(component);
    }
    components
}

fn reachable<I>(node_count: usize, root: usize, successors: impl Fn(usize) -> I) -> Vec<bool>
where
    I: IntoIterator<Item = usize>,
{
    let mut reached = vec![false; node_count];
    let mut queue = VecDeque::from([root]);
    while let Some(node) = queue.pop_front() {
        if reached[node] {
            continue;
        }
        reached[node] = true;
        for successor in successors(node) {
            if !reached[successor] {
                queue.push_back(successor);
            }
        }
    }
    reached
}

fn shortest_cycle(graph: &[Vec<StrongEdge>], component: &[usize]) -> Option<Vec<StrongEdge>> {
    let mut in_component = vec![false; graph.len()];
    for node in component {
        in_component[*node] = true;
    }

    let mut best: Option<Vec<StrongEdge>> = None;
    for &source in component {
        for edge in &graph[source] {
            if !in_component[edge.to] {
                continue;
            }
            let Some(mut tail) = shortest_path(graph, edge.to, edge.from, &in_component) else {
                continue;
            };
            let mut cycle = vec![edge.clone()];
            cycle.append(&mut tail);
            let better = best.as_ref().is_none_or(|current| {
                (cycle.len(), cycle[0].order) < (current.len(), current[0].order)
            });
            if better {
                best = Some(cycle);
            }
        }
    }
    best
}

fn shortest_path(
    graph: &[Vec<StrongEdge>],
    start: usize,
    goal: usize,
    allowed: &[bool],
) -> Option<Vec<StrongEdge>> {
    if start == goal {
        return Some(Vec::new());
    }
    let mut predecessor = vec![None; graph.len()];
    let mut queue = VecDeque::from([start]);
    while let Some(node) = queue.pop_front() {
        for edge in &graph[node] {
            if !allowed[edge.to] || predecessor[edge.to].is_some() || edge.to == start {
                continue;
            }
            predecessor[edge.to] = Some(edge.clone());
            if edge.to == goal {
                let mut path = Vec::new();
                let mut current = goal;
                while current != start {
                    let edge = predecessor[current].clone()?;
                    current = edge.from;
                    path.push(edge);
                }
                path.reverse();
                return Some(path);
            }
            queue.push_back(edge.to);
        }
    }
    None
}

fn report_cycle(types: &TypeTable, cycle: &[StrongEdge], sink: &mut Sink) {
    let first = &cycle[0];
    let path = cycle
        .iter()
        .map(|edge| {
            format!(
                "{}.{}",
                class_name(types, ClassId(edge.field_owner as u32)),
                edge.field
            )
        })
        .chain(std::iter::once(class_name(
            types,
            ClassId(first.from as u32),
        )))
        .collect::<Vec<_>>()
        .join(" -> ");
    let mut diagnostic = Diagnostic::lint(
        codes::L3001,
        first.span,
        format!("potential reference cycle: {path}"),
    )
    .primary_label("this strong field closes the shortest statically visible cycle")
    .note("every edge in this statically visible cycle is strong; the cycle may leak at run time");
    if let Some(edge) = cycle
        .iter()
        .find(|edge| edge.weak_replacement.is_some())
    {
        let replacement = edge
            .weak_replacement
            .as_ref()
            .expect("selected weak replacement edge must have a replacement");
        let owner = class_name(types, ClassId(edge.field_owner as u32));
        diagnostic = diagnostic.suggest(
            format!(
                "replace `{owner}.{}` with `{replacement}` if this is a back-reference",
                edge.field
            ),
            edge.type_span,
            replacement.clone(),
        );
    }
    for edge in cycle.iter().skip(1) {
        let owner = class_name(types, ClassId(edge.field_owner as u32));
        diagnostic = diagnostic.secondary(
            edge.span,
            format!("strong edge `{owner}.{}` is part of this cycle", edge.field),
        );
    }
    sink.emit(diagnostic);
}

fn class_name(types: &TypeTable, class: ClassId) -> String {
    let def = types.class_def(class);
    let Some((name, arguments)) = &def.origin else {
        return def.name.to_string();
    };
    let arguments = arguments
        .iter()
        .map(|argument| types.display(*argument))
        .collect::<Vec<_>>()
        .join(", ");
    format!("{name}[{arguments}]")
}

#[cfg(test)]
mod tests {
    use super::*;
    use ember_span::{Span, Symbol};
    use ember_types::{ClassDef, ClassOpenness, FieldDef, FieldVis, StructDef};

    fn class(types: &mut TypeTable, name: &str) -> ClassId {
        types.add_class(ClassDef {
            name: Symbol::intern(name),
            fields: Vec::new(),
            span: Span::DUMMY,
            openness: ClassOpenness::Final,
            base: None,
            has_drop: false,
            origin: None,
            declaring_module: 0,
        })
    }

    fn field(name: &str, ty: Ty) -> FieldDef {
        FieldDef {
            name: Symbol::intern(name),
            ty,
            span: Span::DUMMY,
            ty_span: Span::DUMMY,
            has_default: false,
            is_let: false,
            read_only_outside: false,
            vis: FieldVis::Private,
        }
    }

    #[test]
    fn a_direct_strong_cycle_warns_once() {
        let (mut types, _) = TypeTable::new();
        let parent = class(&mut types, "Parent");
        let child = class(&mut types, "Child");
        let parent_ty = types.intern(TyKind::Class(parent));
        let child_ty = types.intern(TyKind::Class(child));
        types.class_def_mut(parent).fields = vec![field("child", child_ty)];
        types.class_def_mut(child).fields = vec![field("parent", parent_ty)];

        let mut sink = Sink::new();
        lint_strong_cycles(&types, &mut sink);

        assert_eq!(sink.warning_count(), 1);
        assert_eq!(sink.diagnostics()[0].code, Some(codes::L3001));
        assert!(
            sink.diagnostics()[0]
                .message
                .contains("Parent.child -> Child.parent -> Parent")
        );
    }

    #[test]
    fn a_weak_edge_does_not_form_a_strong_cycle() {
        let (mut types, _) = TypeTable::new();
        let parent = class(&mut types, "Parent");
        let child = class(&mut types, "Child");
        let parent_ty = types.intern(TyKind::Class(parent));
        let child_ty = types.intern(TyKind::Class(child));
        let weak_child = types.add_struct(StructDef {
            name: Symbol::intern("Weak[Child]"),
            fields: Vec::new(),
            span: Span::DUMMY,
            derives_copy: true,
            has_drop: false,
            drops_fields: false,
            origin: Some((Symbol::intern("Weak"), vec![child_ty])),
            declaring_module: 0,
        });
        let weak_child_ty = types.intern(TyKind::Struct(weak_child));
        types.class_def_mut(parent).fields = vec![field("child", weak_child_ty)];
        types.class_def_mut(child).fields = vec![field("parent", parent_ty)];

        let mut sink = Sink::new();
        lint_strong_cycles(&types, &mut sink);

        assert_eq!(sink.warning_count(), 0);
    }

    #[test]
    fn inspection_marks_unresolved_owners_unknown() {
        let (mut types, _) = TypeTable::new();
        let holder = class(&mut types, "Holder");
        let parameter = types.intern(TyKind::Param {
            index: 0,
            name: Symbol::intern("T"),
        });
        types.class_def_mut(holder).fields = vec![field("value", parameter)];

        let inspection = inspect_ownership_graph(&types);

        assert_eq!(inspection.edges.len(), 1);
        assert_eq!(inspection.edges[0].kind, OwnershipEdgeKind::Unknown);
        assert_eq!(inspection.edges[0].target, None);
        assert!(inspection.shortest_cycles.is_empty());
    }
}
