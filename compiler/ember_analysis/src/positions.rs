//! `[BRW-16]` — positions. A name for a list element made by indexing (`boss = ref mut enemies[i]`,
//! `boss = enemies[0]`), of a list reached through locals that own it, parameters, `this` and
//! one-owner fields, keeps the index it was made with, and the element is found again through the
//! list's current buffer after an add: where a call that only adds to the list (`push`, `extend`,
//! `reserve`, `[EFF-24]`) returns and the name is used again, the name is made again from the kept
//! index. So an add ends no borrow the name holds, while what may remove, insert, reorder, clear or
//! replace the list still meets it (`E3060` at the later use, `borrows.rs`).
//!
//! It runs before the borrow check, which then sees the name made again after the add.

use std::collections::{HashMap, HashSet};

use ember_mir::{
    BasicBlock, BasicBlockId, Body, Builtin, FuncRef, LocalDecl, LocalId, LocalKind, Operand, Place, Projection, Rvalue,
    Stmt, StmtKind, Terminator,
};
use ember_types::{TyKind, TypeTable};

pub fn repoint_positions_all(bodies: &mut [Body], types: &TypeTable) {
    for body in bodies {
        repoint_positions(body, types);
    }
}

/// A position name: the list it names an element of, the view each making takes (`&mut xs[_11]`),
/// and where each making is.
struct Position {
    list: Place,
    made: Place,
    mutable: bool,
    makings: Vec<(usize, usize)>,
}

fn repoint_positions(body: &mut Body, types: &TypeTable) {
    let positions = position_names(body, types);
    if positions.is_empty() {
        return;
    }
    let blocks: Vec<usize> = (0..body.blocks.len()).collect();
    let dominators = crate::loop_version::dominators(body, 0, &blocks);
    // Each add of a position's list made after the position on every way there, where the name is
    // used again: the call's block and the name.
    let mut repoints: Vec<(usize, LocalId)> = Vec::new();
    for (&name, position) in &positions {
        for (block, data) in body.blocks.iter().enumerate() {
            let Terminator::Call { func: FuncRef::Builtin { which, .. }, args, next, .. } = &data.terminator else { continue };
            if !matches!(which, Builtin::ArrayPush | Builtin::ArrayExtend | Builtin::ArrayReserve | Builtin::ArrayReserveHint { .. }) {
                continue;
            }
            let Some(Operand::Copy(given) | Operand::Move(given)) = args.first() else { continue };
            if !given.projection.is_empty() || borrowed_place(&data.stmts, given.local).as_ref() != Some(&position.list) {
                continue;
            }
            let made_before = position
                .makings
                .iter()
                .any(|&(at, _)| at == block || dominators.get(&block).is_some_and(|set| set.contains(&at)));
            if made_before && used_again(body, name, next.0 as usize) {
                repoints.push((block, name));
            }
        }
    }
    // The index each making read, kept in a local of the name's own: "the index is read once, where
    // the name is made" (a later change of `i` moves no position). The copies go in after every
    // block is made, last place first, so the places recorded stay right.
    let mut kept: HashMap<LocalId, Place> = HashMap::new();
    let mut copies: Vec<(usize, usize, Vec<Stmt>)> = Vec::new();
    for (block, name) in repoints {
        let position = &positions[&name];
        let again = kept.entry(name).or_insert_with(|| {
            let mut again = position.made.clone();
            let mut saved = Vec::new();
            for projection in &mut again.projection {
                if let Projection::Index(index) = projection {
                    let local = LocalId(body.locals.len() as u32);
                    let decl = &body.locals[index.0 as usize];
                    body.locals.push(LocalDecl { ty: decl.ty, kind: LocalKind::Temp, name: None, span: decl.span });
                    saved.push(local);
                    *index = local;
                }
            }
            for &(making_block, at) in &position.makings {
                let stmt = &body.blocks[making_block].stmts[at];
                let (StmtKind::Assign { rvalue: Rvalue::Ref { place: made, .. }, .. }, span) = (&stmt.kind, stmt.span) else { continue };
                let indexes = made.projection.iter().filter_map(|p| if let Projection::Index(index) = p { Some(*index) } else { None });
                let made_copies = indexes
                    .zip(&saved)
                    .map(|(index, local)| Stmt {
                        kind: StmtKind::Assign { place: whole(*local), rvalue: Rvalue::Use(Operand::Copy(whole(index))) },
                        span,
                    })
                    .collect();
                copies.push((making_block, at + 1, made_copies));
            }
            again
        });
        let (making_block, at) = position.makings[0];
        let span = body.blocks[making_block].stmts[at].span;
        let stmt = Stmt {
            kind: StmtKind::Assign { place: whole(name), rvalue: Rvalue::Ref { place: again.clone(), mutable: position.mutable } },
            span,
        };
        // On the call's own edge: a block that makes the name again, then goes where the call went.
        let bridge = BasicBlockId(body.blocks.len() as u32);
        let terminator_span = body.blocks[block].terminator_span;
        let Terminator::Call { next, .. } = &mut body.blocks[block].terminator else { continue };
        let after = std::mem::replace(next, bridge);
        body.blocks.push(BasicBlock { stmts: vec![stmt], terminator: Terminator::Goto(after), terminator_span, call_span: None });
    }
    copies.sort_by(|a, b| (b.0, b.1).cmp(&(a.0, a.1)));
    for (block, at, stmts) in copies {
        body.blocks[block].stmts.splice(at..at, stmts);
    }
}

/// The names the program wrote whose every making is one view (but for its indexes) of an element
/// of one list by indexing, the list reached through what owns it.
fn position_names(body: &Body, types: &TypeTable) -> HashMap<LocalId, Position> {
    let mut found: HashMap<LocalId, Position> = HashMap::new();
    let mut refused: HashSet<LocalId> = HashSet::new();
    for (block, data) in body.blocks.iter().enumerate() {
        for (at, stmt) in data.stmts.iter().enumerate() {
            let StmtKind::Assign { place, rvalue } = &stmt.kind else { continue };
            if !place.projection.is_empty() || !matches!(body.locals[place.local.0 as usize].kind, LocalKind::User) {
                continue;
            }
            let making = match rvalue {
                Rvalue::Ref { place: made, mutable } => list_of(body, types, made).map(|list| (list, made, *mutable)),
                _ => None,
            };
            match (making, found.get_mut(&place.local)) {
                (Some((list, made, mutable)), Some(position))
                    if position.list == list && position.mutable == mutable && same_shape(&position.made, made) =>
                {
                    position.makings.push((block, at))
                }
                (Some((list, made, mutable)), None) => {
                    found.insert(place.local, Position { list, made: made.clone(), mutable, makings: vec![(block, at)] });
                }
                _ => {
                    refused.insert(place.local);
                }
            }
        }
    }
    found.retain(|name, _| !refused.contains(name));
    found
}

/// `[BRW-16]` — whether a loan of `loan_place` held by `borrower` is a position: a name the program
/// wrote for an element of a list, made by indexing, the list reached through what owns it.
pub(crate) fn is_position(body: &Body, types: &TypeTable, loan_place: &Place, borrower: LocalId) -> bool {
    matches!(body.locals[borrower.0 as usize].kind, LocalKind::User) && list_of(body, types, loan_place).is_some()
}

/// Two views of one shape: the same projections, an index local for an index local.
fn same_shape(a: &Place, b: &Place) -> bool {
    a.local == b.local
        && a.projection.len() == b.projection.len()
        && a.projection.iter().zip(&b.projection).all(|pair| match pair {
            (Projection::Index(_), Projection::Index(_)) => true,
            (x, y) => x == y,
        })
}

/// The list `made` names an element of by indexing, when it is reached through what owns it.
fn list_of(body: &Body, types: &TypeTable, made: &Place) -> Option<Place> {
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
            // A parameter's or `this`'s view of what the caller owns.
            Projection::Deref => n == 0 && matches!(root.kind, LocalKind::Arg) && matches!(types.kind(root.ty), TyKind::Ref { .. }),
            _ => false,
        }
    });
    owned.then_some(list)
}

/// The place the `ref mut` in `taken` holds, where one of `stmts` takes it.
fn borrowed_place(stmts: &[Stmt], taken: LocalId) -> Option<Place> {
    stmts.iter().rev().find_map(|stmt| match &stmt.kind {
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
            if read || made {
                verdict = Some(read);
                break;
            }
        }
        match verdict {
            Some(read) if read => return true,
            Some(_) => continue,
            None => {}
        }
        let mut read = false;
        crate::loop_version::visit_terminator(&data.terminator, &mut |place, _, _| read |= place.local == name);
        if read {
            return true;
        }
        work.extend(crate::borrows::access_successors(&data.terminator).into_iter().map(|b| b.0 as usize));
    }
    false
}

fn whole(local: LocalId) -> Place {
    Place { local, projection: Vec::new() }
}
