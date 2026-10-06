//! `[RC-3]`: a hidden owner may be transferred into another owner only when
//! that owner survives through the hidden owner's original drop. Last use
//! alone is not enough: an owned callee can release its argument early.

use std::collections::HashSet;
use ember_mir::{Body, Builtin, CastKind, FuncRef, LocalId, LocalKind, Operand, Place, Rvalue, StmtKind, Terminator};
use ember_types::{TyKind, TypeTable};
use crate::long_access_lint::successors;

type Point = (usize, usize);

#[derive(Default)]
struct Uses {
    definitions: Vec<Point>,
    copies: Vec<Point>,
    touches: HashSet<Point>,
    drops: HashSet<Point>,
    referenced: bool,
    moved: bool,
    conditional_or_projected_drop: bool,
}

#[derive(Clone, Copy)]
enum Sink {
    Local(LocalId),
    Array,
}

/// This first slice handles fresh non-Sync instances and their temporary
/// upcasts. Unknown dynamic instances and uncounted borrowed handles stay
/// counted. Cache freshness and dominance, then prove each stage's intervals.
pub fn transfer_temporary_owners_all(bodies: &mut [Body], types: &TypeTable) -> usize {
    bodies.iter_mut().map(|body| transfer(body, types)).sum()
}

fn transfer(body: &mut Body, types: &TypeTable) -> usize {
    if !body.locals.iter().any(|local| local.kind == LocalKind::Temp
        && matches!(types.kind(local.ty), TyKind::Class(_) | TyKind::ClassInterface(_))) { return 0; }
    let edges: Vec<Vec<usize>> = body.blocks.iter().map(|block| successors(&block.terminator)).collect();
    let mut budget = IntervalBudget::for_body(body, &edges);
    transfer_with_budget(body, types, &edges, &mut budget)
}

fn transfer_with_budget(body: &mut Body, types: &TypeTable, edges: &[Vec<usize>], budget: &mut IntervalBudget) -> usize {
    let dominators = Dominators::new(edges);
    let mut fresh = FreshPlain::new(body.locals.len());
    let mut uncounted = vec![false; body.locals.len()];
    for local in &body.uncounted_handles { uncounted[local.0 as usize] = true; }
    let mut changed = 0;
    // Local transfers first: they can remove a pure temporary drop between
    // an Array push and its argument's drop. Recompute uses before pushes,
    // so their survival intervals describe the changed ownership chain.
    for arrays in [false, true] {
        let uses = uses(body);
        let mut candidates = Vec::new();
        for (index, info) in uses.iter().enumerate() {
            let source = LocalId(index as u32);
            if body.local(source).kind != LocalKind::Temp
                || uncounted[index]
                || info.definitions.len() != 1 || info.copies.len() != 1
                || info.drops.len() != 1 || info.referenced || info.moved
                || info.conditional_or_projected_drop
                || !fresh_plain(body, types, &uses, source, &mut fresh)
            {
                continue;
            }
            let copy = info.copies[0];
            let Some(sink) = sink(body, types, &uses, &uncounted, source, copy) else { continue };
            if arrays != matches!(sink, Sink::Array) { continue; }
            // The definition dominates the copy, and a normal path from the
            // definition cannot end this owner or skip its one consumer.
            let definition = info.definitions[0];
            if !dominators.dominates(definition, copy)
                || !interval(body, after(body, definition), copy, budget, |point| {
                    !info.drops.contains(&point) && !storage_end(body, point, source)
                })
            {
                continue;
            }
            let drop = *info.drops.iter().next().expect("one source drop");
            if !interval(body, after(body, copy), drop, budget, |point| {
                if info.touches.contains(&point) || info.definitions.contains(&point) || storage_end(body, point, source) {
                    return false;
                }
                match sink {
                    Sink::Local(owner) => {
                        let target = &uses[owner.0 as usize];
                        !target.definitions.contains(&point) && !target.drops.contains(&point)
                            && !storage_end(body, point, owner)
                    }
                    // A push takes ownership into the list. Before the old
                    // drop, no call, destructor or store may empty/repoint
                    // that list, including through an unrelated alias.
                    Sink::Array => quiet(body, types, point),
                }
            }) {
                continue;
            }
            candidates.push((source, copy, drop));
        }
        // Local edges compose: each replacement owner survives the previous
        // owner's old drop, so a chain extends that guarantee transitively.
        // Array intervals admit no intervening ownership change. Recompute
        // uses between these two kinds, especially their removed drops.
        for (source, copy, drop) in candidates {
            move_copy(body, source, copy);
            body.blocks[drop.0].stmts[drop.1].kind = StmtKind::Nop;
            changed += 1;
        }
    }
    changed
}

fn operand(uses: &mut [Uses], value: &Operand, point: Point) {
    if let Operand::Copy(place) | Operand::Move(place) = value {
        let info = &mut uses[place.local.0 as usize];
        info.touches.insert(point);
        if matches!(value, Operand::Move(_)) { info.moved = true; }
        else if place.projection.is_empty() { info.copies.push(point); }
        for projection in &place.projection {
            if let ember_mir::Projection::Index(local) = projection {
                uses[local.0 as usize].touches.insert(point);
            }
        }
    }
}

fn place(uses: &mut [Uses], target: &Place, point: Point, definition: bool) {
    let info = &mut uses[target.local.0 as usize];
    if definition && target.projection.is_empty() { info.definitions.push(point); }
    else { info.touches.insert(point); }
    for projection in &target.projection {
        if let ember_mir::Projection::Index(local) = projection {
            uses[local.0 as usize].touches.insert(point);
        }
    }
}

fn uses(body: &Body) -> Vec<Uses> {
    let mut result: Vec<Uses> = (0..body.locals.len()).map(|_| Uses::default()).collect();
    for (b, block) in body.blocks.iter().enumerate() {
        for (s, statement) in block.stmts.iter().enumerate() {
            let point = (b, s);
            match &statement.kind {
                StmtKind::Assign { place: target, rvalue } => {
                    place(&mut result, target, point, true);
                    match rvalue {
                        Rvalue::Use(value) | Rvalue::UnaryOp { operand: value, .. }
                        | Rvalue::Cast { operand: value, .. } => operand(&mut result, value, point),
                        Rvalue::BinaryOp { lhs, rhs, .. } => {
                            operand(&mut result, lhs, point); operand(&mut result, rhs, point);
                        }
                        Rvalue::Aggregate { operands, .. } => for value in operands { operand(&mut result, value, point); },
                        Rvalue::Repeat { value, .. } => operand(&mut result, value, point),
                        Rvalue::Ref { place: target, .. } => {
                            result[target.local.0 as usize].referenced = true;
                            place(&mut result, target, point, false);
                        }
                        Rvalue::Discriminant(target) => place(&mut result, target, point, false),
                    }
                }
                StmtKind::CheckedBinaryOp { dest, overflow, lhs, rhs, .. } => {
                    place(&mut result, dest, point, true); place(&mut result, overflow, point, true);
                    operand(&mut result, lhs, point); operand(&mut result, rhs, point);
                }
                StmtKind::Drop { place: target, flag, .. } => {
                    let info = &mut result[target.local.0 as usize];
                    info.drops.insert(point);
                    info.conditional_or_projected_drop |= flag.is_some() || !target.projection.is_empty();
                    place(&mut result, target, point, false);
                }
                StmtKind::BeginAccess { place: target, .. } | StmtKind::EndAccess { place: target, .. }
                | StmtKind::BeginAccessTransfer { place: target, .. } | StmtKind::EndAccessTransfer { place: target, .. } => {
                    place(&mut result, target, point, false);
                }
                StmtKind::StorageLive(_) | StmtKind::StorageDead(_) | StmtKind::Nop => {}
            }
        }
        let point = (b, block.stmts.len());
        match &block.terminator {
            Terminator::Call { func, args, dest, .. } => {
                place(&mut result, dest, point, true);
                for value in args { operand(&mut result, value, point); }
                if let FuncRef::Indirect { operand: value, .. } = func { operand(&mut result, value, point); }
            }
            Terminator::SwitchInt { discr, .. } => operand(&mut result, discr, point),
            Terminator::Assert { cond, msg, .. } => {
                operand(&mut result, cond, point);
                match msg {
                    ember_mir::AssertKind::Bounds { len, index } => {
                        operand(&mut result, len, point); operand(&mut result, index, point);
                    }
                    ember_mir::AssertKind::RefCellBorrow { file, line } => {
                        operand(&mut result, file, point); operand(&mut result, line, point);
                    }
                    ember_mir::AssertKind::Panic { message } => operand(&mut result, message, point),
                    _ => {}
                }
            }
            Terminator::Return => { result[0].touches.insert(point); }
            Terminator::Goto(_) | Terminator::Unreachable => {}
        }
    }
    result
}

fn copied_source(rvalue: &Rvalue) -> Option<LocalId> {
    let value = match rvalue {
        Rvalue::Use(value) | Rvalue::Cast { kind: CastKind::ClassUpcast | CastKind::ClassInterfaceUpcast { .. }, operand: value, .. } => value,
        _ => return None,
    };
    match value {
        Operand::Copy(place) | Operand::Move(place) if place.projection.is_empty() => Some(place.local),
        _ => None,
    }
}

#[derive(Clone, Copy)]
enum Origin {
    Plain,
    Other,
    Local(usize),
}

#[derive(Clone, Copy)]
enum FreshState {
    Unknown,
    Visiting,
    Resolved(bool),
}

/// Copy/upcast ancestry is unchanged by this pass: replacing Copy with Move
/// preserves its source, and removed drops do not change definitions or refs.
/// Cache it across both stages; each origin is examined once, including cycles.
struct FreshPlain {
    states: Vec<FreshState>,
}

impl FreshPlain {
    fn new(locals: usize) -> Self { Self { states: vec![FreshState::Unknown; locals] } }

    fn resolve(&mut self, local: usize, mut origin: impl FnMut(usize) -> Origin) -> bool {
        let mut path = Vec::new();
        let mut current = local;
        let plain = loop {
            match self.states[current] {
                FreshState::Resolved(plain) => break plain,
                FreshState::Visiting => break false,
                FreshState::Unknown => {}
            }
            self.states[current] = FreshState::Visiting;
            path.push(current);
            match origin(current) {
                Origin::Plain => break true,
                Origin::Other => break false,
                Origin::Local(source) => current = source,
            }
        };
        for local in path { self.states[local] = FreshState::Resolved(plain); }
        plain
    }
}

fn fresh_plain(body: &Body, types: &TypeTable, uses: &[Uses], local: LocalId, cache: &mut FreshPlain) -> bool {
    cache.resolve(local.0 as usize, |index| {
        let current = LocalId(index as u32);
        if uses[index].definitions.len() != 1
            || body.local(current).kind != LocalKind::Temp || uses[current.0 as usize].referenced
            || !matches!(types.kind(body.local(current).ty), TyKind::Class(_) | TyKind::ClassInterface(_))
        { return Origin::Other; }
        let (b, s) = uses[index].definitions[0];
        if s == body.blocks[b].stmts.len() {
            return if matches!(&body.blocks[b].terminator,
                Terminator::Call { func: FuncRef::Builtin { which: Builtin::ClassNew { class_id, .. }, .. }, .. }
                if !types.class_def(*class_id).is_sync)
            { Origin::Plain } else { Origin::Other };
        }
        let StmtKind::Assign { rvalue, .. } = &body.blocks[b].stmts[s].kind else { return Origin::Other };
        copied_source(rvalue).map_or(Origin::Other, |source| Origin::Local(source.0 as usize))
    })
}

fn sink(body: &Body, types: &TypeTable, uses: &[Uses], uncounted: &[bool], source: LocalId, point: Point) -> Option<Sink> {
    let block = &body.blocks[point.0];
    if point.1 < block.stmts.len() {
        if let StmtKind::Assign { place: dest, rvalue } = &block.stmts[point.1].kind
            && dest.projection.is_empty() && copied_source(rvalue) == Some(source)
            && body.local(dest.local).kind == LocalKind::Temp
            && matches!(types.kind(body.local(dest.local).ty), TyKind::Class(_) | TyKind::ClassInterface(_))
            && !uncounted[dest.local.0 as usize]
        {
            let info = &uses[dest.local.0 as usize];
            return (info.definitions.len() == 1 && !info.referenced && !info.moved
                && !info.conditional_or_projected_drop).then_some(Sink::Local(dest.local));
        }
    } else if let Terminator::Call { func: FuncRef::Builtin { which: Builtin::ArrayPush, .. }, args, .. } = &block.terminator
        && args.len() == 2 && matches!(&args[1], Operand::Copy(place) if place.local == source && place.projection.is_empty())
    { return Some(Sink::Array); }
    None
}

fn next(body: &Body, point: Point) -> Vec<Point> {
    let block = &body.blocks[point.0];
    if point.1 < block.stmts.len() { vec![(point.0, point.1 + 1)] }
    else { successors(&block.terminator).into_iter().map(|b| (b, 0)).collect() }
}

fn after(body: &Body, point: Point) -> Vec<Point> { next(body, point) }

fn storage_end(body: &Body, point: Point, local: LocalId) -> bool {
    body.blocks[point.0].stmts.get(point.1).is_some_and(|s|
        matches!(s.kind, StmtKind::StorageLive(id) | StmtKind::StorageDead(id) if id == local))
}

/// Immediate block dominators in reverse postorder, then intervals in their
/// tree. This uses linear storage and makes each point query constant-time;
/// searching from entry per candidate revisits a long function's prefixes.
struct Dominators {
    ranges: Vec<Option<(usize, usize)>>,
    #[cfg(test)]
    work: usize,
}

impl Dominators {
    fn new(edges: &[Vec<usize>]) -> Self {
        let count = edges.len();
        if count == 0 {
            return Self { ranges: Vec::new(), #[cfg(test)] work: 0 };
        }
        #[cfg(test)]
        let mut work = 0;
        // Iterative DFS keeps deeply nested expressions off the Rust stack.
        let mut seen = vec![false; count];
        let mut order = Vec::new();
        let mut stack = vec![(0, 0)];
        seen[0] = true;
        while let Some((block, next)) = stack.pop() {
            if let Some(&successor) = edges[block].get(next) {
                #[cfg(test)]
                { work += 1; }
                stack.push((block, next + 1));
                if !seen[successor] {
                    seen[successor] = true;
                    stack.push((successor, 0));
                }
            } else {
                order.push(block);
            }
        }
        order.reverse();
        let mut position = vec![usize::MAX; count];
        let mut predecessors = vec![Vec::new(); count];
        for (rank, &block) in order.iter().enumerate() {
            position[block] = rank;
            for &successor in &edges[block] {
                #[cfg(test)]
                { work += 1; }
                predecessors[successor].push(block);
            }
        }
        let mut parent = vec![None; count];
        parent[0] = Some(0);
        loop {
            let mut changed = false;
            for &block in order.iter().skip(1) {
                let mut common: Option<usize> = None;
                for &predecessor in &predecessors[block] {
                    #[cfg(test)]
                    { work += 1; }
                    if parent[predecessor].is_none() { continue; }
                    common = Some(match common {
                        None => predecessor,
                        Some(mut left) => {
                            let mut right = predecessor;
                            // Known parents point earlier in reverse postorder.
                            // Walk up to the nearest common dominator.
                            while left != right {
                                #[cfg(test)]
                                { work += 1; }
                                if position[left] > position[right] {
                                    left = parent[left].expect("known dominator parent");
                                } else {
                                    right = parent[right].expect("known dominator parent");
                                }
                            }
                            left
                        }
                    });
                }
                if parent[block] != common {
                    parent[block] = common;
                    changed = true;
                }
            }
            if !changed { break; }
        }
        let mut children = vec![Vec::new(); count];
        for &block in order.iter().skip(1) {
            children[parent[block].expect("reachable block has a dominator")].push(block);
        }
        let mut ranges: Vec<Option<(usize, usize)>> = vec![None; count];
        let mut clock = 0;
        let mut stack = vec![(0, false)];
        while let Some((block, leave)) = stack.pop() {
            if leave {
                ranges[block].as_mut().expect("entered dominator tree node").1 = clock;
            } else {
                ranges[block] = Some((clock, 0));
                clock += 1;
                stack.push((block, true));
                stack.extend(children[block].iter().rev().map(|&child| (child, false)));
            }
        }
        Self { ranges, #[cfg(test)] work }
    }

    fn dominates(&self, definition: Point, use_point: Point) -> bool {
        let (Some((entry, exit)), Some((use_entry, _))) =
            (self.ranges[definition.0], self.ranges[use_point.0]) else { return false };
        if definition.0 == use_point.0 { definition.1 < use_point.1 }
        else { entry <= use_entry && use_entry < exit }
    }
}

/// A shared bound on interval proof work, including successor edges. Long,
/// overlapping owners must not repeatedly walk the whole body. Exhaustion
/// declines unproved transfers; the count operations remain as before.
/// This bounds these walks, not the dominator algorithm's worst-case work.
struct IntervalBudget {
    remaining: usize,
    #[cfg(test)]
    spent: usize,
}

impl IntervalBudget {
    fn for_body(body: &Body, edges: &[Vec<usize>]) -> Self {
        let points = body.blocks.iter().fold(body.blocks.len(), |total, block|
            total.saturating_add(block.stmts.len()));
        let size = edges.iter().fold(points, |total, successors|
            total.saturating_add(successors.len()));
        Self { remaining: size.saturating_mul(32), #[cfg(test)] spent: 0 }
    }

    fn spend(&mut self, amount: usize) -> bool {
        let charged = amount.min(self.remaining);
        self.remaining -= charged;
        #[cfg(test)]
        { self.spent += charged; }
        charged == amount
    }
}

/// All normal paths must reach `end`. The predicate must preserve the
/// replacement owner across any calls or panic callbacks in the interval.
/// Reject cycles rather than infer an unproved progress rank. Iterative DFS
/// avoids exhausting the compiler stack on large expressions.
fn interval(body: &Body, starts: Vec<Point>, end: Point, budget: &mut IntervalBudget, allowed: impl Fn(Point) -> bool) -> bool {
    if !budget.spend(starts.len().max(1)) { return false; }
    let mut done = HashSet::new();
    let mut active = HashSet::new();
    let mut stack: Vec<_> = starts.into_iter().map(|point| (point, false)).collect();
    while let Some((point, leave)) = stack.pop() {
        if !budget.spend(1) { return false; }
        if point == end || done.contains(&point) { continue; }
        if leave { active.remove(&point); done.insert(point); continue; }
        if !active.insert(point) || !allowed(point) { return false; }
        let block = &body.blocks[point.0];
        if point.1 == block.stmts.len() && matches!(block.terminator, Terminator::Return) { return false; }
        let successors = next(body, point);
        if !budget.spend(successors.len()) { return false; }
        stack.push((point, true));
        stack.extend(successors.into_iter().map(|point| (point, false)));
    }
    true
}

fn quiet(body: &Body, types: &TypeTable, point: Point) -> bool {
    let block = &body.blocks[point.0];
    if let Some(stmt) = block.stmts.get(point.1) {
        return match &stmt.kind {
            StmtKind::Nop => true,
            StmtKind::StorageLive(local) | StmtKind::StorageDead(local) => !types.needs_drop(body.local(*local).ty),
            StmtKind::Assign { place, .. } => place.projection.is_empty()
                && matches!(types.kind(body.local(place.local).ty), TyKind::Bool | TyKind::Int(_) | TyKind::Uint(_) | TyKind::Float(_)),
            StmtKind::CheckedBinaryOp { dest, overflow, .. } => dest.projection.is_empty() && overflow.projection.is_empty(),
            _ => false,
        };
    }
    // A failed Assert invokes the embedding panic callback before aborting.
    // It could clear an aliased list, so it is not a quiet ownership interval.
    !matches!(block.terminator, Terminator::Call { .. } | Terminator::Assert { .. })
}

fn move_copy(body: &mut Body, source: LocalId, point: Point) {
    let convert = |operand: &mut Operand| {
        if matches!(operand, Operand::Copy(place) if place.local == source && place.projection.is_empty())
            && let Operand::Copy(place) = operand
        { *operand = Operand::Move(place.clone()); }
    };
    let block = &mut body.blocks[point.0];
    if let Some(stmt) = block.stmts.get_mut(point.1) {
        match &mut stmt.kind {
            StmtKind::Assign { rvalue: Rvalue::Use(value) | Rvalue::Cast { operand: value, .. }, .. } => convert(value),
            _ => unreachable!("proved local ownership sink"),
        }
    } else if let Terminator::Call { args, .. } = &mut block.terminator {
        convert(&mut args[1]);
    } else { unreachable!("proved Array ownership sink"); }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ember_mir::{BasicBlock, BasicBlockId, Const, LocalDecl, Stmt};
    use ember_span::{Span, Symbol};
    use ember_types::{ClassDef, ClassId, ClassOpenness, Ty};

    fn owner_body(pairs: usize) -> (Body, TypeTable, ClassId) {
        let (mut types, common) = TypeTable::new();
        let class = types.add_class(ClassDef {
            name: Symbol::intern("Owner"), fields: Vec::new(), span: Span::DUMMY,
            is_sync: false, openness: ClassOpenness::Final, base: None,
            has_drop: false, origin: None, declaring_module: 0,
        });
        let owner_ty = types.intern(TyKind::Class(class));
        let mut locals = vec![LocalDecl {
            ty: common.void, kind: LocalKind::Return, name: None, span: Span::DUMMY,
        }];
        locals.extend((0..2 * pairs).map(|_| LocalDecl {
            ty: owner_ty, kind: LocalKind::Temp, name: None, span: Span::DUMMY,
        }));
        let body = Body {
            name: "owners".to_string(), symbol: "owners".to_string(),
            is_unsafe: false, abi: None, overflow: ember_types::OverflowPolicy::Panic,
            fp: ember_types::FpMode::Strict, inline: ember_mir::InlineHint::default(),
            export_thread_policy: ember_mir::ExportThreadPolicy::Any,
            locals, blocks: Vec::new(), arg_count: 0, param_modes: Vec::new(),
            span: Span::DUMMY, borrows: None, sources: Vec::new(), is_lambda: false,
            emit_if_used: false, borrowed_params: Vec::new(), for_iterators: Vec::new(),
            call_argument_bindings: Vec::new(), callable_regions: None,
            closure_environment: None, closure_captures_by_move: false,
            class_owner: None, class_virtual_slot: None, is_abstract: false,
            is_extern_declaration: false, ffi_counted: None, mut_self: false,
            elided_accesses: Vec::new(), hoisted_accesses: Vec::new(),
            uncounted_handles: Vec::new(), removed_checks: Vec::new(), restrict_views: false, stack_buffers: Vec::new(),
        };
        (body, types, class)
    }

    fn block(stmts: Vec<StmtKind>, terminator: Terminator) -> BasicBlock {
        BasicBlock {
            stmts: stmts.into_iter().map(|kind| Stmt::new(kind, Span::DUMMY)).collect(),
            terminator, terminator_span: Span::DUMMY,
        }
    }

    fn allocate(class: ClassId, ty: Ty, owner: u32, next: u32) -> Terminator {
        Terminator::Call {
            func: FuncRef::Builtin { which: Builtin::ClassNew { class_id: class, init: None }, arg_ty: ty },
            args: Vec::new(), dest: Place::local(LocalId(owner)), next: BasicBlockId(next),
        }
    }

    fn copy_owner(source: u32, owner: u32) -> StmtKind {
        StmtKind::Assign {
            place: Place::local(LocalId(owner)),
            rvalue: Rvalue::Use(Operand::Copy(Place::local(LocalId(source)))),
        }
    }

    fn drop_owner(owner: u32) -> StmtKind {
        StmtKind::Drop { place: Place::local(LocalId(owner)), flag: None, scope_end: true }
    }

    fn branch(yes: u32, no: u32) -> Terminator {
        Terminator::SwitchInt {
            discr: Operand::Const(Const::Bool(true)),
            targets: vec![(1, BasicBlockId(yes))], otherwise: BasicBlockId(no),
        }
    }

    fn assert_original_owner(body: &Body, source: u32, copy: Point, drop: Point) {
        assert!(matches!(&body.blocks[copy.0].stmts[copy.1].kind,
            StmtKind::Assign { rvalue: Rvalue::Use(Operand::Copy(place)), .. }
            if place.local == LocalId(source)));
        assert!(matches!(&body.blocks[drop.0].stmts[drop.1].kind,
            StmtKind::Drop { place, .. } if place.local == LocalId(source)));
    }

    fn reaches(edges: &[Vec<usize>], target: usize, excluded: Option<usize>) -> bool {
        let mut seen = vec![false; edges.len()];
        let mut stack = vec![0];
        while let Some(block) = stack.pop() {
            if excluded == Some(block) || seen[block] { continue; }
            if block == target { return true; }
            seen[block] = true;
            stack.extend(edges[block].iter().copied());
        }
        false
    }

    #[test]
    fn dominance_matches_path_removal_on_all_three_block_graphs() {
        // Independent oracle, including cycles, duplicate entry paths and
        // unreachable blocks: removing a dominator prevents reaching a use.
        for mask in 0usize..(1 << 9) {
            let edges: Vec<Vec<usize>> = (0..3).map(|from|
                (0..3).filter(|&to| mask & (1 << (from * 3 + to)) != 0).collect()
            ).collect();
            let dominators = Dominators::new(&edges);
            for definition in 0..3 {
                for use_block in 0..3 {
                    let expected = reaches(&edges, use_block, None)
                        && (definition == use_block || !reaches(&edges, use_block, Some(definition)));
                    assert_eq!(dominators.dominates((definition, 0), (use_block, 1)), expected,
                        "edges {edges:?}, definition {definition}, use {use_block}");
                }
            }
        }
    }

    #[test]
    fn dominance_respects_statement_order_and_ignores_unreachable_predecessors() {
        let edges = vec![vec![1, 2], vec![3], vec![3], vec![4], vec![], vec![3]];
        let dominators = Dominators::new(&edges);
        assert!(dominators.dominates((0, 8), (3, 0)));
        assert!(dominators.dominates((3, 0), (4, 0)));
        assert!(!dominators.dominates((1, 0), (3, 0)));
        assert!(!dominators.dominates((5, 0), (3, 0)));
        assert!(!dominators.dominates((5, 0), (5, 1)));
        assert!(dominators.dominates((3, 1), (3, 2)));
        assert!(!dominators.dominates((3, 2), (3, 1)));
        assert!(!dominators.dominates((3, 1), (3, 1)));
    }

    #[test]
    fn irreducible_cfg_updates_a_parent_after_a_later_predecessor_is_known() {
        let edges = vec![vec![1, 2], vec![3], vec![3, 5], vec![4], vec![2], vec![]];
        let dominators = Dominators::new(&edges);
        // The first DFS reaches 3 through 1, then discovers its other entry
        // through 2. Neither entry alone dominates the cycle's shared blocks.
        assert!(!dominators.dominates((1, 0), (3, 0)));
        assert!(!dominators.dominates((2, 0), (3, 0)));
        assert!(dominators.dominates((3, 0), (4, 0)));
        assert!(dominators.dominates((2, 0), (5, 0)));
        assert!(!dominators.dominates((3, 0), (5, 0)));
    }

    #[test]
    fn long_cfg_chain_builds_with_linear_work_and_constant_space_per_block() {
        let count = 16_384;
        let edges: Vec<Vec<usize>> = (0..count).map(|block|
            if block + 1 < count { vec![block + 1] } else { vec![] }
        ).collect();
        let dominators = Dominators::new(&edges);
        assert_eq!(dominators.ranges.len(), count);
        assert!(dominators.work <= 8 * count, "{} operations for {count} blocks", dominators.work);
        for block in 0..count {
            assert!(dominators.dominates((0, 0), (block, 1)));
            assert!(dominators.dominates((block, 0), (count - 1, 1)));
        }
    }

    #[test]
    fn long_fresh_chain_resolves_each_origin_once() {
        let count = 16_384;
        let mut fresh = FreshPlain::new(count);
        let mut examined = 0;
        for local in 0..count {
            assert!(fresh.resolve(local, |current| {
                examined += 1;
                if current + 1 < count { Origin::Local(current + 1) } else { Origin::Plain }
            }));
        }
        assert_eq!(examined, count);
    }

    #[test]
    fn provenance_rejects_cycles_and_unknown_roots_without_poisoning_other_chains() {
        let origins = [Origin::Local(1), Origin::Local(2), Origin::Local(1),
            Origin::Local(2), Origin::Other, Origin::Local(4), Origin::Plain, Origin::Local(6)];
        let mut fresh = FreshPlain::new(origins.len());
        let mut examined = 0;
        for local in 0..origins.len() {
            assert_eq!(fresh.resolve(local, |current| { examined += 1; origins[current] }), local >= 6);
        }
        assert_eq!(examined, origins.len());
    }

    #[test]
    fn provenance_follows_previously_verified_move_edges() {
        assert_eq!(copied_source(&Rvalue::Use(Operand::Move(Place::local(LocalId(7))))), Some(LocalId(7)));
    }

    #[test]
    fn a_return_path_cannot_skip_the_one_consumer() {
        let (mut body, types, class) = owner_body(1);
        let ty = body.local(LocalId(1)).ty;
        body.blocks = vec![
            block(vec![], allocate(class, ty, 1, 1)),
            block(vec![], branch(2, 3)),
            block(vec![copy_owner(1, 2), drop_owner(1), drop_owner(2)], Terminator::Return),
            block(vec![], Terminator::Return),
        ];
        assert_eq!(transfer(&mut body, &types), 0);
        assert_original_owner(&body, 1, (2, 0), (2, 1));
    }

    #[test]
    fn a_return_path_cannot_skip_the_original_drop() {
        let (mut body, types, class) = owner_body(1);
        let ty = body.local(LocalId(1)).ty;
        body.blocks = vec![
            block(vec![], allocate(class, ty, 1, 1)),
            block(vec![copy_owner(1, 2)], branch(2, 3)),
            block(vec![drop_owner(1), drop_owner(2)], Terminator::Return),
            block(vec![], Terminator::Return),
        ];
        assert_eq!(transfer(&mut body, &types), 0);
        assert_original_owner(&body, 1, (1, 0), (2, 0));
    }

    #[test]
    fn an_early_replacement_drop_cannot_end_the_original_owner() {
        let (mut body, types, class) = owner_body(1);
        let ty = body.local(LocalId(1)).ty;
        body.blocks = vec![
            block(vec![], allocate(class, ty, 1, 1)),
            block(vec![copy_owner(1, 2), drop_owner(2), drop_owner(1)], Terminator::Return),
        ];
        assert_eq!(transfer(&mut body, &types), 0);
        assert_original_owner(&body, 1, (1, 0), (1, 2));
    }

    #[test]
    fn a_cycle_cannot_be_assumed_to_reach_the_original_drop() {
        let (mut body, types, class) = owner_body(1);
        let ty = body.local(LocalId(1)).ty;
        body.blocks = vec![
            block(vec![], allocate(class, ty, 1, 1)),
            block(vec![copy_owner(1, 2)], branch(2, 3)),
            block(vec![], Terminator::Goto(BasicBlockId(2))),
            block(vec![drop_owner(1), drop_owner(2)], Terminator::Return),
        ];
        assert_eq!(transfer(&mut body, &types), 0);
        assert_original_owner(&body, 1, (1, 0), (3, 0));
    }

    #[test]
    fn uncounted_sources_and_destinations_keep_the_original_operations() {
        for uncounted in [1, 2] {
            let (mut body, types, class) = owner_body(1);
            let ty = body.local(LocalId(1)).ty;
            body.blocks = vec![
                block(vec![], allocate(class, ty, 1, 1)),
                block(vec![copy_owner(1, 2), drop_owner(1), drop_owner(2)], Terminator::Return),
            ];
            body.uncounted_handles.push(LocalId(uncounted));
            assert_eq!(transfer(&mut body, &types), 0);
            assert_original_owner(&body, 1, (1, 0), (1, 1));
        }
    }

    #[test]
    fn an_array_transfer_rejects_a_panic_callback_interval() {
        let (mut body, mut types, class) = owner_body(1);
        let ty = body.local(LocalId(1)).ty;
        let array_ty = types.intern(TyKind::Vec { elem: ty, text: false });
        body.locals[2].ty = array_ty;
        body.locals[2].kind = LocalKind::User;
        body.blocks = vec![
            block(vec![], Terminator::Call {
                func: FuncRef::Builtin { which: Builtin::ArrayNew, arg_ty: array_ty },
                args: Vec::new(), dest: Place::local(LocalId(2)), next: BasicBlockId(1),
            }),
            block(vec![], allocate(class, ty, 1, 2)),
            block(vec![], Terminator::Call {
                func: FuncRef::Builtin { which: Builtin::ArrayPush, arg_ty: ty },
                args: vec![Operand::Copy(Place::local(LocalId(2))), Operand::Copy(Place::local(LocalId(1)))],
                dest: Place::local(LocalId(0)), next: BasicBlockId(3),
            }),
            block(vec![], Terminator::Assert {
                cond: Operand::Const(Const::Bool(true)), expected: true,
                msg: ember_mir::AssertKind::Overflow(ember_mir::BinOp::Add),
                next: BasicBlockId(4), span: Span::DUMMY,
            }),
            block(vec![drop_owner(1), drop_owner(2)], Terminator::Return),
        ];
        assert_eq!(transfer(&mut body, &types), 0);
        assert!(matches!(&body.blocks[2].terminator,
            Terminator::Call { args, .. } if matches!(args[1], Operand::Copy(_))));
        assert!(matches!(body.blocks[4].stmts[0].kind, StmtKind::Drop { .. }));
        // The same owner transfers across an ordinary CFG edge.
        body.blocks[3].terminator = Terminator::Goto(BasicBlockId(4));
        assert_eq!(transfer(&mut body, &types), 1);
        assert!(matches!(&body.blocks[2].terminator,
            Terminator::Call { args, .. } if matches!(args[1], Operand::Move(_))));
        assert!(matches!(body.blocks[4].stmts[0].kind, StmtKind::Nop));
    }

    #[test]
    fn overlapping_owners_share_one_bounded_interval_budget() {
        let count = 1_024;
        let (mut body, types, class) = owner_body(count);
        let ty = body.local(LocalId(1)).ty;
        for pair in 0..count {
            body.blocks.push(block(vec![], allocate(class, ty, (2 * pair + 1) as u32, (pair + 1) as u32)));
        }
        let mut stmts: Vec<_> = (0..count).map(|pair|
            copy_owner((2 * pair + 1) as u32, (2 * pair + 2) as u32)).collect();
        stmts.extend((0..count).map(|pair| drop_owner((2 * pair + 1) as u32)));
        stmts.extend((0..count).map(|pair| drop_owner((2 * pair + 2) as u32)));
        body.blocks.push(block(stmts, Terminator::Return));
        let edges: Vec<_> = body.blocks.iter().map(|block| successors(&block.terminator)).collect();
        let mut budget = IntervalBudget::for_body(&body, &edges);
        let limit = budget.remaining;
        let changed = transfer_with_budget(&mut body, &types, &edges, &mut budget);
        assert!(changed > 0 && changed < count, "{changed} overlapping transfers");
        assert_eq!(budget.remaining, 0);
        assert_eq!(budget.spent, limit);
        for pair in 0..count {
            match &body.blocks[count].stmts[pair].kind {
                StmtKind::Assign { rvalue: Rvalue::Use(Operand::Move(_)), .. } =>
                    assert!(matches!(body.blocks[count].stmts[count + pair].kind, StmtKind::Nop)),
                _ => assert_original_owner(&body, (2 * pair + 1) as u32,
                    (count, pair), (count, count + pair)),
            }
        }
    }

    #[test]
    fn many_short_owner_intervals_still_all_transfer() {
        let count = 1_024;
        let (mut body, types, class) = owner_body(count);
        let ty = body.local(LocalId(1)).ty;
        body.blocks.push(block(vec![], allocate(class, ty, 1, 1)));
        for pair in 0..count {
            let source = (2 * pair + 1) as u32;
            let owner = source + 1;
            let terminator = if pair + 1 < count {
                allocate(class, ty, source + 2, (pair + 2) as u32)
            } else { Terminator::Return };
            body.blocks.push(block(vec![copy_owner(source, owner), drop_owner(source), drop_owner(owner)], terminator));
        }
        let edges: Vec<_> = body.blocks.iter().map(|block| successors(&block.terminator)).collect();
        let mut budget = IntervalBudget::for_body(&body, &edges);
        let limit = budget.remaining;
        assert_eq!(transfer_with_budget(&mut body, &types, &edges, &mut budget), count);
        assert!(budget.spent <= 8 * count && budget.spent < limit);
    }
}
