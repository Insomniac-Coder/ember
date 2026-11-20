//! `[BRW-16]` — positions. A name for a list element made by indexing (`boss = ref mut enemies[i]`,
//! `boss = enemies[0]`), of a list reached through locals that own it, parameters, `this` and
//! one-owner fields, keeps the index it was made with, and the element is found again through the
//! list's current buffer after an add: where a call that only adds to the list (`push`, `extend`,
//! `reserve`, `[EFF-24]`) returns and the name is used again, the name is made again from the kept
//! index. So an add ends no borrow the name holds, while what may remove, insert, reorder, clear or
//! replace the list still meets it (`E3060` at the later use, `borrows.rs`).
//!
//! One rule says where: [`repoints`]. The borrow check reads it to let those adds meet the name's
//! borrow (the borrow is still live across them, so anything else that may reshape the list before
//! the name's last use is refused), and the pass runs after the checks, with the same rule.

use std::collections::{HashMap, HashSet};

use ember_mir::{
    BasicBlock, BasicBlockId, Body, Builtin, Const, FuncRef, LocalDecl, LocalId, LocalKind, Operand, Place, Projection, Rvalue,
    Stmt, StmtKind, Terminator,
};
use ember_types::{TyKind, TypeTable};

/// Each function's summary by symbol (`[EFF-24]`): what a written function only adds to.
pub type Summaries = HashMap<String, crate::summaries::Summary>;

/// Makes the positions of every body again after their adds, by the summaries the borrow check read
/// (`BorrowFacts::summaries`), so the rule is the one it checked. A body made so already (a bridge's
/// making comes before any read of the name) is left as it is.
pub fn repoint_positions_all(bodies: &mut [Body], types: &TypeTable, summaries: Option<&Summaries>) {
    for body in bodies {
        repoint_positions(body, types, summaries);
    }
}

/// A making of a position: the list, the view taken (`&mut xs[_11]`) and its word.
#[derive(Clone)]
struct Making {
    list: Place,
    made: Place,
    mutable: bool,
}

impl Making {
    /// The same list, word and view but for its index locals.
    fn same_as(&self, other: &Making) -> bool {
        self.list == other.list && self.mutable == other.mutable && same_shape(&self.made, &other.made)
    }
}

/// A whole write of a name the program wrote: its block, its statement (the block's length for a
/// call's result), and the making it is when it makes a position.
struct Write {
    block: usize,
    at: usize,
    making: Option<Making>,
}

/// Every whole write of each name the program wrote that some statement makes a position of, but for
/// a name the body takes a view of (`ref mut boss`: what it is given may make it point anywhere).
fn writes(body: &Body, types: &TypeTable, roots: &HashMap<LocalId, LocalId>) -> HashMap<LocalId, Vec<Write>> {
    let mut found: HashMap<LocalId, Vec<Write>> = HashMap::new();
    let mut viewed: HashSet<LocalId> = HashSet::new();
    let named = |local: LocalId| matches!(body.locals[local.0 as usize].kind, LocalKind::User);
    for (block, data) in body.blocks.iter().enumerate() {
        for (at, stmt) in data.stmts.iter().enumerate() {
            match &stmt.kind {
                StmtKind::Assign { place, rvalue } => {
                    // A `ref mut` of the name itself may make it point anywhere; a read-only view of it (a
                    // lambda's capture) reads the name, which is made again after an add.
                    if let Rvalue::Ref { place: taken, mutable: true } = rvalue
                        && taken.projection.is_empty()
                    {
                        viewed.insert(taken.local);
                    }
                    if place.projection.is_empty() && named(place.local) {
                        let making = match rvalue {
                            Rvalue::Ref { place: made, mutable } => {
                                list_of(body, types, roots, made).map(|list| Making { list, made: made.clone(), mutable: *mutable })
                            }
                            _ => None,
                        };
                        found.entry(place.local).or_default().push(Write { block, at, making });
                    }
                }
                StmtKind::Drop { place, .. } if place.projection.is_empty() && named(place.local) => {
                    found.entry(place.local).or_default().push(Write { block, at, making: None });
                }
                _ => {}
            }
        }
        if let Terminator::Call { dest, .. } = &data.terminator
            && dest.projection.is_empty()
            && named(dest.local)
        {
            found.entry(dest.local).or_default().push(Write { block, at: data.stmts.len(), making: None });
        }
    }
    found.retain(|name, writes| !viewed.contains(name) && writes.iter().any(|write| write.making.is_some()));
    found
}

/// The making of a position of `list` that a name holds at the end of `call`'s statements on every
/// way there, one shape for all of them: the last whole write of the name on each way back must be a
/// making of that list, and no way may reach the body's start without one.
fn reaching_making(writes: &[Write], predecessors: &[Vec<usize>], call: usize, call_stmts: usize, list: &Place) -> Option<Making> {
    // The last write in a block, before `limit`.
    let last = |block: usize, limit: usize| writes.iter().filter(|w| w.block == block && w.at < limit).max_by_key(|w| w.at);
    let mut found: Option<Making> = None;
    let mut agree = |write: &Write| -> bool {
        let Some(making) = &write.making else { return false };
        if &making.list != list {
            return false;
        }
        match &found {
            Some(first) => first.same_as(making),
            None => {
                found = Some(making.clone());
                true
            }
        }
    };
    if let Some(write) = last(call, call_stmts) {
        return if agree(write) { found } else { None };
    }
    if predecessors[call].is_empty() {
        return None;
    }
    let mut seen = HashSet::new();
    let mut work: Vec<usize> = predecessors[call].clone();
    while let Some(block) = work.pop() {
        if !seen.insert(block) {
            continue;
        }
        if let Some(write) = last(block, usize::MAX) {
            if !agree(write) {
                return None;
            }
            continue;
        }
        if predecessors[block].is_empty() {
            return None;
        }
        work.extend(predecessors[block].iter().copied());
    }
    found
}

fn predecessors(body: &Body) -> Vec<Vec<usize>> {
    let mut predecessors = vec![Vec::new(); body.blocks.len()];
    for (block, data) in body.blocks.iter().enumerate() {
        for next in crate::borrows::access_successors(&data.terminator) {
            predecessors[next.0 as usize].push(block);
        }
    }
    predecessors
}

/// Each add a position is made again after: the call's block, the name, and the making to make again.
fn plan(body: &Body, types: &TypeTable, summaries: Option<&Summaries>) -> Vec<(usize, LocalId, Making)> {
    let roots = crate::borrows::module_variable_roots(body);
    let writes = writes(body, types, &roots);
    let mut repoints = Vec::new();
    if writes.is_empty() {
        return repoints;
    }
    let predecessors = predecessors(body);
    let mut names: Vec<LocalId> = writes.keys().copied().collect();
    names.sort();
    for (block, data) in body.blocks.iter().enumerate() {
        let Terminator::Call { func, args, next, dest } = &data.terminator else { continue };
        // `[BRW-16]` (review 4) — a written function whose result may hold a view (into the list it is given)
        // keeps no position across it: the result would be a second name of the element.
        let summaries = if matches!(func, FuncRef::Builtin { .. }) || !types.may_be_view(crate::regions::place_type(body, types, dest)) { summaries } else { None };
        for argument in added_arguments(func, args, summaries) {
            let Some(Operand::Copy(given) | Operand::Move(given)) = args.get(argument) else { continue };
            if !given.projection.is_empty() {
                continue;
            }
            let Some(list) = borrowed_place(body, given.local) else { continue };
            let list = module_variable_root(&roots, list);
            for &name in &names {
                // The name is not given to the call nor written by it (`spawn(ref mut es, boss)`,
                // `boss = pick(ref mut es)`, `boss.x = f(ref mut es)`): the callee would use, or the result
                // be written through, the address the add may move; the call meets the borrow there.
                let named = |place: &Place| place.local == name;
                if named(dest) || args.iter().any(|arg| matches!(arg, Operand::Copy(place) | Operand::Move(place) if named(place))) {
                    continue;
                }
                if let Some(making) = reaching_making(&writes[&name], &predecessors, block, data.stmts.len(), &list)
                    && used_again(body, name, next.0 as usize)
                    && !repoints.iter().any(|(at, other, _): &(usize, LocalId, Making)| *at == block && *other == name)
                {
                    repoints.push((block, name, making));
                }
            }
        }
    }
    repoints
}

/// The adds a position is made again after: each `(the call's block, the name)`, where the call only
/// adds to the name's list, the name holds a position of it on every way there, and is used again.
pub(crate) fn repoints(body: &Body, types: &TypeTable, summaries: Option<&Summaries>) -> HashSet<(usize, LocalId)> {
    plan(body, types, summaries).into_iter().map(|(block, name, _)| (block, name)).collect()
}

/// `[EFF-24]` — the arguments a call only adds to what they point at: a built-in that only adds, its
/// first; a written function, each its summary only adds to.
fn added_arguments(func: &FuncRef, args: &[Operand], summaries: Option<&Summaries>) -> Vec<usize> {
    if only_adds(func) {
        return vec![0];
    }
    let (FuncRef::Direct { symbol, .. }, Some(summaries)) = (func, summaries) else { return Vec::new() };
    let Some(summary) = summaries.get(symbol) else { return Vec::new() };
    (0..args.len())
        .filter(|&param| summary.only_adds(&crate::summaries::Reach { root: crate::summaries::Root::Param(param), path: vec![crate::summaries::Step::Deref] }))
        .collect()
}

/// Whether the access at `(block, index)` is the `ref mut` of its list that an add in `repoints`
/// takes, an add after which `borrower` is made again: the call the borrowed temporary is given.
pub(crate) fn repointed_add_at(body: &Body, block: usize, index: usize, borrower: LocalId, repoints: &HashSet<(usize, LocalId)>) -> Option<usize> {
    let Some(StmtKind::Assign { place: taken, rvalue: Rvalue::Ref { mutable: true, .. } }) =
        body.blocks[block].stmts.get(index).map(|stmt| &stmt.kind)
    else {
        return None;
    };
    if !taken.projection.is_empty() {
        return None;
    }
    body.blocks.iter().enumerate().find_map(|(call, data)| match &data.terminator {
        Terminator::Call { args, .. }
            if args.iter().any(|arg| matches!(arg, Operand::Copy(given) | Operand::Move(given) if given.local == taken.local && given.projection.is_empty()))
                && repoints.contains(&(call, borrower)) =>
        {
            Some(call)
        }
        _ => None,
    })
}

/// `[EFF-24]` — a call that only adds to the list its first argument points at: the built-ins, and
/// `extend(owns items)`, which the compiler routes to std's `extend_owned`.
pub(crate) fn only_adds(func: &FuncRef) -> bool {
    match func {
        FuncRef::Builtin { which, .. } => {
            matches!(which, Builtin::ArrayPush | Builtin::ArrayExtend | Builtin::ArrayReserve | Builtin::ArrayReserveHint { .. })
        }
        FuncRef::Direct { symbol, .. } => {
            ember_branding::demangle(symbol).is_some_and(|found| found.components == ["std", "core", "Array", "extend_owned"])
        }
        _ => false,
    }
}

fn repoint_positions(body: &mut Body, types: &TypeTable, summaries: Option<&Summaries>) {
    let repoints = plan(body, types, summaries);
    if repoints.is_empty() {
        return;
    }
    // The index each making read, kept in a local of the name's own: "the index is read once, where
    // the name is made" (a later change of `i` moves no position). One set of locals for each name and
    // making shape, set by every making of that shape; the copies go in last place first, after every
    // block is made, so the places recorded stay right.
    let writes = writes(body, types, &crate::borrows::module_variable_roots(body));
    let usize_ty = types.find(&TyKind::Uint(ember_types::UintTy::Usize)).expect("`usize` is interned");
    let mut kept: Vec<(LocalId, Making, Place)> = Vec::new();
    let mut copies: Vec<(usize, usize, Vec<Stmt>)> = Vec::new();
    for (block, name, making) in repoints {
        let again = match kept.iter().find(|(kept_name, kept_making, _)| *kept_name == name && kept_making.same_as(&making)) {
            Some((_, _, again)) => again.clone(),
            None => {
                let mut again = making.made.clone();
                let mut saved = Vec::new();
                let span = body.locals[name.0 as usize].span;
                for projection in &mut again.projection {
                    if matches!(projection, Projection::Index(_) | Projection::ConstIndex(_)) {
                        let local = LocalId(body.locals.len() as u32);
                        body.locals.push(LocalDecl { ty: usize_ty, kind: LocalKind::Temp, name: None, span });
                        saved.push(local);
                        *projection = Projection::Index(local);
                    }
                }
                for write in &writes[&name] {
                    let Some(this) = &write.making else { continue };
                    if !this.same_as(&making) {
                        continue;
                    }
                    let span = body.blocks[write.block].stmts[write.at].span;
                    // Each index the making read: a local's value, or the constant it wrote.
                    let indexes = this.made.projection.iter().filter_map(|p| match p {
                        Projection::Index(index) => Some(Operand::Copy(whole(*index))),
                        Projection::ConstIndex(value) => Some(Operand::Const(Const::Int { value: *value as u128, ty: usize_ty })),
                        _ => None,
                    });
                    let made_copies = indexes
                        .zip(&saved)
                        .map(|(index, local)| Stmt { kind: StmtKind::Assign { place: whole(*local), rvalue: Rvalue::Use(index) }, span })
                        .collect();
                    copies.push((write.block, write.at + 1, made_copies));
                }
                kept.push((name, making.clone(), again.clone()));
                again
            }
        };
        let span = writes[&name]
            .iter()
            .find(|w| w.making.as_ref().is_some_and(|m| m.same_as(&making)))
            .map_or(body.blocks[block].terminator_span, |w| body.blocks[w.block].stmts[w.at].span);
        let stmt = Stmt { kind: StmtKind::Assign { place: whole(name), rvalue: Rvalue::Ref { place: again, mutable: making.mutable } }, span };
        // On the call's own edge: a block that makes the name again, then goes where the call went.
        let bridge = BasicBlockId(body.blocks.len() as u32);
        let terminator_span = body.blocks[block].terminator_span;
        let Terminator::Call { next, .. } = &mut body.blocks[block].terminator else { continue };
        let after = std::mem::replace(next, bridge);
        body.blocks.push(BasicBlock { stmts: vec![stmt], terminator: Terminator::Goto(after), terminator_span, call_span: None });
    }
    copies.sort_by(|a, b| (b.0, b.1).cmp(&(a.0, a.1)));
    copies.dedup_by(|a, b| a.0 == b.0 && a.1 == b.1);
    for (block, at, stmts) in copies {
        body.blocks[block].stmts.splice(at..at, stmts);
    }
}

/// `[BRW-16]` — whether a loan of `loan_place` held by `borrower` is a position: a name the program
/// wrote for an element of a list, made by indexing, the list reached through what owns it.
pub(crate) fn is_position(body: &Body, types: &TypeTable, roots: &HashMap<LocalId, LocalId>, loan_place: &Place, borrower: LocalId) -> bool {
    matches!(body.locals[borrower.0 as usize].kind, LocalKind::User) && list_of(body, types, roots, loan_place).is_some()
}

/// Two views of one shape: the same projections, an index local for an index local.
fn same_shape(a: &Place, b: &Place) -> bool {
    a.local == b.local
        && a.projection.len() == b.projection.len()
        && a.projection.iter().zip(&b.projection).all(|pair| match pair {
            (Projection::Index(_) | Projection::ConstIndex(_), Projection::Index(_) | Projection::ConstIndex(_)) => true,
            (x, y) => x == y,
        })
}

/// The list `made` names an element of by indexing, when it is reached through what owns it: a
/// module variable's at its static's one root (`[LT-3]`).
fn list_of(body: &Body, types: &TypeTable, roots: &HashMap<LocalId, LocalId>, made: &Place) -> Option<Place> {
    let at = made.projection.iter().position(|p| matches!(p, Projection::Index(_) | Projection::ConstIndex(_)))?;
    let list = Place { local: made.local, projection: made.projection[..at].to_vec() };
    if !matches!(types.kind(crate::regions::place_type(body, types, &list)), TyKind::Vec { text: false, .. }) {
        return None;
    }
    let root = &body.locals[made.local.0 as usize];
    let owned = list.projection.iter().enumerate().all(|(n, p)| {
        // A list reached through a link (`shares`, `shares mut`, `sharedref mut`) gives addresses
        // (`[BRW-17]`).
        let from = Place { local: made.local, projection: list.projection[..n].to_vec() };
        if matches!(types.kind(crate::regions::place_type(body, types, &from)), TyKind::Link { .. }) {
            return false;
        }
        match p {
            Projection::Field(_) => true,
            // A parameter's or `this`'s view of what the caller owns, or a module variable's own
            // storage (`static mut`, `[LT-3]`).
            Projection::Deref => {
                n == 0
                    && matches!(types.kind(root.ty), TyKind::Ref { .. })
                    && (matches!(root.kind, LocalKind::Arg) || roots.contains_key(&made.local))
            }
            _ => false,
        }
    });
    owned.then(|| module_variable_root(roots, list))
}

/// A place through a module variable's temporary, at its static's one root.
fn module_variable_root(roots: &HashMap<LocalId, LocalId>, place: Place) -> Place {
    match roots.get(&place.local) {
        Some(&root) if place.projection.first() == Some(&Projection::Deref) => Place { local: root, ..place },
        _ => place,
    }
}

/// The place the `ref mut` in `taken` holds, where the body takes it (the compiler's temporary is
/// made once, perhaps before the call's arguments).
fn borrowed_place(body: &Body, taken: LocalId) -> Option<Place> {
    body.blocks.iter().flat_map(|block| block.stmts.iter()).find_map(|stmt| match &stmt.kind {
        StmtKind::Assign { place, rvalue: Rvalue::Ref { place: borrowed, mutable: true } }
            if place.local == taken && place.projection.is_empty() =>
        {
            Some(borrowed.clone())
        }
        _ => None,
    })
}

/// Whether `name` may be read from the start of block `from` on before it is made again.
fn used_again(body: &Body, name: LocalId, from: usize) -> bool {
    next_read(body, name, from).is_some()
}

/// Where `name` may next be read from the start of block `from` on, before it is made again.
pub(crate) fn next_read(body: &Body, name: LocalId, from: usize) -> Option<ember_span::Span> {
    let mut seen = HashSet::new();
    let mut work = vec![from];
    while let Some(block) = work.pop() {
        if !seen.insert(block) {
            continue;
        }
        let data = &body.blocks[block];
        let mut verdict = None;
        for stmt in &data.stmts {
            let (mut read, mut made) = (false, false);
            crate::loop_version::visit_stmt(stmt, &mut |place, written, _| {
                if place.local == name {
                    if written && place.projection.is_empty() {
                        made = true;
                    } else {
                        read = true;
                    }
                }
            });
            if read {
                return Some(stmt.span);
            }
            if made {
                verdict = Some(());
                break;
            }
        }
        if verdict.is_some() {
            continue;
        }
        let mut read = false;
        crate::loop_version::visit_terminator(&data.terminator, &mut |place, _, _| read |= place.local == name);
        if read {
            return Some(data.terminator_span);
        }
        work.extend(crate::borrows::access_successors(&data.terminator).into_iter().map(|b| b.0 as usize));
    }
    None
}

fn whole(local: LocalId) -> Place {
    Place { local, projection: Vec::new() }
}
