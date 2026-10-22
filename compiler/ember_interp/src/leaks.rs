//! `[WK-4]`, `[WK-8]`, `[WK-15]` — the debug leak report at the end of a run, as the runtime
//! writes it (`debug_leak_report`): the objects still alive, their ownership edges as the
//! compiler's enumerators list them (`emit_debug_edge_enumerators`), the strongly connected
//! components of the strong edges, and each one that is a cycle printed with its objects, its
//! edges, the edge to weaken and the shortest cycle through each object.

use ember_types::{Ty, TyKind};

use crate::object::{payload_offset, shared_inner, weak_inner, Info};
use crate::{Interp, R};

const LIST_LIMIT: usize = 64;
const VISIT_LIMIT: usize = 65536;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Strong,
    Weak,
    Unknown,
}

struct Edge {
    from: usize,
    to: Option<usize>,
    field: String,
    weak_replacement: Option<String>,
    kind: Kind,
    statically_predicted: bool,
}

/// A pointer as C's `%p` prints it.
fn pointer(address: u64) -> String {
    if cfg!(windows) { format!("{address:016X}") } else { format!("{address:#x}") }
}

impl Interp<'_, '_> {
    /// Each edge the value of `ty` at `addr` holds, as `debug_edge_lines` enumerates them: the
    /// target object (none for an unknown one) and its kind.
    fn value_edges(&self, addr: u64, ty: Ty, out: &mut Vec<(Option<u64>, Kind, bool)>, top: bool) -> R<()> {
        match self.types.kind(ty) {
            TyKind::Class(_) => out.push((Some(self.mem.read_u64(addr)?), Kind::Strong, top)),
            TyKind::ClassInterface(_) => out.push((Some(self.mem.read_u64(addr)?), Kind::Strong, false)),
            TyKind::Vec { elem, .. } => {
                let (ptr, len) = (self.mem.read_u64(addr)?, self.mem.read_u64(addr + 8)?);
                if ptr != 0 {
                    let step = self.size(*elem);
                    for i in 0..len {
                        self.value_edges(ptr + i * step, *elem, out, top)?;
                    }
                }
            }
            TyKind::Struct(id) => {
                if shared_inner(self.types, *id).is_some() {
                    out.push((Some(self.mem.read_u64(addr)?), Kind::Strong, top));
                    return Ok(());
                }
                if weak_inner(self.types, *id).is_some() {
                    out.push((Some(self.mem.read_u64(addr)?), Kind::Weak, true));
                    return Ok(());
                }
                if let Some(inner) = self.types.compiler_box_inner(*id) {
                    let data = self.mem.read_u64(addr)?;
                    if matches!(self.types.kind(inner), TyKind::Dyn { .. }) {
                        if data != 0 {
                            out.push((None, Kind::Unknown, false));
                        }
                    } else if data != 0 {
                        self.value_edges(data, inner, out, top)?;
                    }
                    return Ok(());
                }
                let offsets = self.layout(ty).field_offsets.clone();
                for (field, offset) in self.types.struct_def(*id).fields.iter().zip(offsets) {
                    self.value_edges(addr + offset, field.ty, out, top)?;
                }
            }
            TyKind::Enum(id) => {
                let def = self.types.enum_def(*id);
                if def.is_unit_only() {
                    return Ok(());
                }
                let tag = self.discriminant(crate::Loc::at(addr, ty))?;
                let Some(variant) = def.variants.iter().position(|v| v.discriminant == tag) else { return Ok(()) };
                let niche = self.types.option_niche(*id).is_some();
                let offsets = if niche { Vec::new() } else { self.types.variant_field_offsets(*id).swap_remove(variant) };
                for (index, field) in def.variants[variant].fields.iter().enumerate() {
                    let at = if niche { addr } else { addr + offsets[index] };
                    self.value_edges(at, field.ty, out, top)?;
                }
            }
            TyKind::Tuple(items) => {
                let offsets = self.layout(ty).field_offsets.clone();
                for (item, offset) in items.iter().zip(offsets) {
                    self.value_edges(addr + offset, *item, out, top)?;
                }
            }
            TyKind::Array { elem, len } => {
                let step = self.size(*elem);
                for i in 0..*len {
                    self.value_edges(addr + i * step, *elem, out, top)?;
                }
            }
            TyKind::Dyn { .. } | TyKind::Param { .. } | TyKind::Assoc { .. } => out.push((None, Kind::Unknown, false)),
            _ => {}
        }
        Ok(())
    }

    /// An object's edges: a class's fields, each labelled `Owner.field`; a `Shared` value's one.
    fn object_edges(&mut self, object: u64, from: usize, index_of: &impl Fn(u64) -> Option<usize>, edges: &mut Vec<Edge>) -> R<()> {
        let mut push = |found: Vec<(Option<u64>, Kind, bool)>, field: &str, replacement: Option<String>| {
            for (target, kind, statically_predicted) in found {
                let weak_replacement = if kind == Kind::Strong && statically_predicted { replacement.clone() } else { None };
                edges.push(Edge {
                    from,
                    to: target.and_then(index_of),
                    field: field.to_string(),
                    weak_replacement,
                    kind,
                    statically_predicted,
                });
            }
        };
        match self.object_info(object)? {
            Info::Class(id) => {
                let layout = self.class_layout(id);
                for index in 0..self.types.class_field_count(id) {
                    let Some((owner, field)) = self.types.class_field_at_info(id, index) else { continue };
                    let label = format!("{}.{}", self.types.class_def(owner).name, field.name);
                    let replacement = match self.types.kind(field.ty) {
                        TyKind::Class(target) if self.types.class_def(owner).origin.is_none() => {
                            Some(format!("Weak[{}]", self.types.class_def(*target).name))
                        }
                        _ => None,
                    };
                    let mut found = Vec::new();
                    self.value_edges(object + layout.fields[index], field.ty, &mut found, true)?;
                    push(found, &label, replacement);
                }
            }
            Info::Shared(id) => {
                let inner = shared_inner(self.types, id).expect("a shared value's type");
                let label = format!("Shared[{}].value", self.types.display(inner));
                let mut found = Vec::new();
                self.value_edges(object + payload_offset(self.layout(inner).align), inner, &mut found, false)?;
                push(found, &label, None);
            }
        }
        Ok(())
    }

    fn leak_name(&self, object: u64) -> String {
        match self.object_info(object) {
            Ok(Info::Class(id)) => self.types.class_def(id).name.to_string(),
            Ok(Info::Shared(id)) => {
                let inner = shared_inner(self.types, id).map_or_else(String::new, |inner| self.types.display(inner));
                let sync = self.types.struct_def(id).origin.as_ref().is_some_and(|(name, _)| name.is("SyncShared"));
                format!("{}[{inner}]", if sync { "SyncShared" } else { "Shared" })
            }
            Err(_) => "<unknown>".to_string(),
        }
    }

    /// `debug_leak_report`: the text, empty when no cycle leaked.
    pub(crate) fn leak_report(&mut self) -> R<String> {
        let mut live: Vec<(u64, u64)> = Vec::new();
        for (&object, &sequence) in &self.objects.live {
            if self.mem.read_u32(object)? != 0 {
                live.push((sequence, object));
            }
        }
        // Newest first.
        live.sort_by(|a, b| b.0.cmp(&a.0));
        let objects: Vec<u64> = live.iter().map(|(_, object)| *object).collect();
        let count = objects.len();
        if count == 0 {
            return Ok(String::new());
        }
        let position: crate::memory::WordMap<u64, usize> = objects.iter().enumerate().map(|(k, o)| (*o, k)).collect();
        let index_of = |object: u64| position.get(&object).copied();
        let mut edges = Vec::new();
        for (k, &object) in objects.iter().enumerate() {
            self.object_edges(object, k, &index_of, &mut edges)?;
        }
        let mut by_object: Vec<Vec<usize>> = vec![Vec::new(); count];
        for (e, edge) in edges.iter().enumerate() {
            by_object[edge.from].push(e);
        }
        let component_of = components(count, &edges, &by_object);
        let mut out = String::new();
        let mut printed = vec![false; count];
        for k in 0..count {
            let c = component_of[k];
            if printed[c] {
                continue;
            }
            printed[c] = true;
            let members: Vec<usize> = (0..count).filter(|&m| component_of[m] == c).collect();
            let cycle = members.len() > 1
                || by_object[k].iter().any(|&e| edges[e].kind == Kind::Strong && edges[e].to == Some(k));
            if cycle {
                self.print_component(&mut out, &objects, &edges, &by_object, &component_of, c, &members);
            }
        }
        Ok(out)
    }

    #[allow(clippy::too_many_arguments)]
    fn print_component(
        &self,
        out: &mut String,
        objects: &[u64],
        edges: &[Edge],
        by_object: &[Vec<usize>],
        component_of: &[usize],
        component: usize,
        members: &[usize],
    ) {
        let internal = |edge: &Edge| edge.kind == Kind::Strong && edge.to.is_some_and(|to| component_of[to] == component);
        let named = |k: usize| format!("{}@{}", self.leak_name(objects[k]), pointer(objects[k]));
        let mut strong = 0;
        let mut statically_predicted = false;
        let (mut suggestion, mut first) = (None, None);
        for &m in members {
            for &e in &by_object[m] {
                let edge = &edges[e];
                if !internal(edge) {
                    continue;
                }
                strong += 1;
                statically_predicted |= edge.statically_predicted;
                first.get_or_insert(e);
                if suggestion.is_none() && edge.weak_replacement.is_some() {
                    suggestion = Some(e);
                }
            }
        }
        out.push_str("warning[L3017]: reference cycle detected\nruntime ownership cycle:\n  objects:");
        for &m in members.iter().take(LIST_LIMIT) {
            out.push_str(&format!(" {}", named(m)));
        }
        if members.len() > LIST_LIMIT {
            out.push_str(&format!(" and {} more", members.len() - LIST_LIMIT));
        }
        out.push_str(&format!("\n  strong edges: {strong}\n  statically predicted: {}\n", if statically_predicted { "yes" } else { "no" }));
        match (suggestion, first) {
            (Some(e), _) => out.push_str(&format!(
                "  suggested weak edge: {} (replace with {})\n",
                edges[e].field,
                edges[e].weak_replacement.as_deref().unwrap_or_default()
            )),
            (None, Some(e)) => out.push_str(&format!(
                "  suggested weak edge: {} (no automatic rewrite: its type is not a plain handle)\n",
                edges[e].field
            )),
            _ => out.push_str("  suggested weak edge: none\n"),
        }
        out.push_str("  edges:\n");
        let mut listed = 0;
        for &m in members {
            for &e in &by_object[m] {
                let edge = &edges[e];
                if edge.kind == Kind::Strong && !internal(edge) {
                    continue;
                }
                listed += 1;
                if listed > LIST_LIMIT {
                    continue;
                }
                let kind = match edge.kind {
                    Kind::Strong => "strong",
                    Kind::Weak => "weak",
                    Kind::Unknown => "unknown",
                };
                let to = edge.to.map_or_else(|| "<unknown>".to_string(), named);
                out.push_str(&format!("    {kind} {}: {} -> {to}\n", edge.field, named(edge.from)));
            }
        }
        if listed > LIST_LIMIT {
            out.push_str(&format!("    and {} more\n", listed - LIST_LIMIT));
        }
        // `[WK-4]` — the shortest strong cycle through each object, by shape.
        let traced = members.len().min(LIST_LIMIT);
        let mut shapes: Vec<(String, Vec<usize>)> = Vec::new();
        let mut too_long = 0;
        for &root in &members[..traced] {
            let Some(path) = shortest_cycle(root, edges, by_object, &internal) else {
                too_long += 1;
                continue;
            };
            match shapes.iter_mut().find(|(p, _)| *p == path) {
                Some((_, sample)) => sample.push(root),
                None => shapes.push((path, vec![root])),
            }
        }
        out.push_str("  shortest cycles:\n");
        for (path, sample) in &shapes {
            out.push_str(&format!("    {path}:"));
            for (i, &k) in sample.iter().take(3).enumerate() {
                out.push_str(&format!("{} {}", if i == 0 { "" } else { "," }, named(k)));
            }
            if sample.len() > 3 {
                out.push_str(&format!(" and {} more", sample.len() - 3));
            }
            out.push('\n');
        }
        if too_long > 0 {
            out.push_str(&format!("    ({too_long} objects on cycles longer than the search follows)\n"));
        }
        if members.len() > traced {
            out.push_str(&format!("    ({} more objects not traced)\n", members.len() - traced));
        }
    }

    /// `[WK-15]` — the class an object is; for the registry of live objects.
    pub(crate) fn track_object(&mut self, object: u64) {
        if self.leak_check {
            self.objects.sequence += 1;
            self.objects.live.insert(object, self.objects.sequence);
        }
    }

    pub(crate) fn untrack_object(&mut self, object: u64) {
        if self.leak_check {
            self.objects.live.remove(&object);
        }
    }
}

/// `debug_components`: the strongly connected components of the strong edges (Tarjan's,
/// numbered as they complete).
fn components(count: usize, edges: &[Edge], by_object: &[Vec<usize>]) -> Vec<usize> {
    let mut number = vec![usize::MAX; count];
    let mut low = vec![0; count];
    let mut on_stack = vec![false; count];
    let mut stack = Vec::new();
    let mut component = vec![0; count];
    let (mut next, mut found) = (0, 0);
    for root in 0..count {
        if number[root] != usize::MAX {
            continue;
        }
        number[root] = next;
        low[root] = next;
        next += 1;
        stack.push(root);
        on_stack[root] = true;
        let mut calls: Vec<(usize, usize)> = vec![(root, 0)];
        while let Some(&(v, at)) = calls.last() {
            if at < by_object[v].len() {
                calls.last_mut().expect("a call").1 += 1;
                let edge = &edges[by_object[v][at]];
                let (Kind::Strong, Some(w)) = (edge.kind, edge.to) else { continue };
                if number[w] == usize::MAX {
                    number[w] = next;
                    low[w] = next;
                    next += 1;
                    stack.push(w);
                    on_stack[w] = true;
                    calls.push((w, 0));
                } else if on_stack[w] && number[w] < low[v] {
                    low[v] = number[w];
                }
                continue;
            }
            if low[v] == number[v] {
                loop {
                    let w = stack.pop().expect("a member");
                    on_stack[w] = false;
                    component[w] = found;
                    if w == v {
                        break;
                    }
                }
                found += 1;
            }
            calls.pop();
            if let Some(&(parent, _)) = calls.last()
                && low[v] < low[parent]
            {
                low[parent] = low[v];
            }
        }
    }
    component
}

/// `debug_shortest_cycle`: the edges of the shortest strong cycle through `root`, by field.
fn shortest_cycle(root: usize, edges: &[Edge], by_object: &[Vec<usize>], internal: &impl Fn(&Edge) -> bool) -> Option<String> {
    let mut parent: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
    let mut seen = std::collections::HashSet::from([root]);
    let mut queue = std::collections::VecDeque::from([root]);
    let mut visited = 0;
    let mut closing = None;
    'search: while let Some(v) = queue.pop_front() {
        visited += 1;
        if visited > VISIT_LIMIT {
            break;
        }
        for &e in &by_object[v] {
            let edge = &edges[e];
            if !internal(edge) {
                continue;
            }
            let to = edge.to.expect("an internal edge has a target");
            if to == root {
                closing = Some(e);
                break 'search;
            }
            if seen.insert(to) {
                parent.insert(to, e);
                queue.push_back(to);
            }
        }
    }
    let mut path = Vec::new();
    let mut e = closing?;
    loop {
        path.push(edges[e].field.as_str());
        if edges[e].from == root {
            break;
        }
        e = parent[&edges[e].from];
    }
    path.reverse();
    Some(path.join(" -> "))
}
