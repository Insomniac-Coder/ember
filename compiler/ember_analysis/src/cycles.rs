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
    span: Span,
    order: usize,
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
                    span: field.span,
                    order,
                });
                order += 1;
            }
        }
    }
    graph
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
                types.class_def(ClassId(edge.field_owner as u32)).name,
                edge.field
            )
        })
        .chain(std::iter::once(
            types.class_def(ClassId(first.from as u32)).name.to_string(),
        ))
        .collect::<Vec<_>>()
        .join(" -> ");
    let owner = types.class_def(ClassId(first.field_owner as u32)).name;
    let target = types.class_def(ClassId(first.to as u32)).name;
    let mut diagnostic = Diagnostic::lint(
        codes::L3001,
        first.span,
        format!("potential reference cycle: {path}"),
    )
    .primary_label("this strong field closes the shortest statically visible cycle")
    .help(format!(
        "use `Weak[{target}]` for `{owner}.{}` if this is a back-reference",
        first.field
    ))
    .note("every edge in this statically visible cycle is strong; the cycle may leak at run time");
    for edge in cycle.iter().skip(1) {
        let owner = types.class_def(ClassId(edge.field_owner as u32)).name;
        diagnostic = diagnostic.secondary(
            edge.span,
            format!("strong edge `{owner}.{}` is part of this cycle", edge.field),
        );
    }
    sink.emit(diagnostic);
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
            has_default: false,
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
}
