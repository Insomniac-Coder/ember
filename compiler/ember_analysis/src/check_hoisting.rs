//! `[EXC-8]`, `[PHIL-5]` — a dynamic exclusivity check every turn of a loop
//! makes, on a handle the loop never changes, runs once before the loop.
//!
//! An access begun and ended back to back is a check: it reads the object's
//! (or field's) access word and changes nothing. Its answer changes only when
//! some access begins or ends on that object. In a loop that begins and ends
//! none (only such checks), drops nothing and calls nothing but list and text
//! built-ins that run no Ember code, every turn's check reads the same word
//! and gives the first turn's answer. When the check is also the first thing
//! each turn does that anything could see (only pure local statements before
//! it, on the one path in), it moves to a guard before the loop: the loop's
//! own test, computed again into fresh locals, and when the loop will run,
//! the check. A loop that never runs checks nothing, as before; one whose
//! first check fails panics at the same point in the same state.

use std::collections::{BTreeMap, HashSet};

use ember_mir::{
    BasicBlock, BasicBlockId, Body, Builtin, FuncRef, HoistedAccess, HoistedAccessProof, LocalId, LocalKind, Operand,
    Place, Rvalue, Stmt, StmtKind, Terminator,
};
use ember_types::{TyKind, TypeTable};

use crate::range_facts::{loop_headers, natural_loops, reverse_postorder};

/// Hoist every loop-invariant access check in `bodies`; returns how many.
pub fn hoist_invariant_checks_all(bodies: &mut [Body], types: &TypeTable) -> usize {
    bodies.iter_mut().map(|body| hoist_invariant_checks(body, types)).sum()
}

fn hoist_invariant_checks(body: &mut Body, types: &TypeTable) -> usize {
    let mut count = 0;
    // One loop at a time: each change adds blocks, so the structure is found
    // again. A hoisted check leaves the loop, so no loop is chosen twice.
    while let Some(plan) = find(body, types) {
        count += plan.checks.len();
        apply(body, plan);
    }
    count
}

struct Plan {
    header: usize,
    /// The one block outside the loop that enters it.
    entry: usize,
    /// The header's successor inside the loop, and the one leaving it.
    inside: usize,
    exit: usize,
    /// `(block, statement)` of each check's begin, its end right after it.
    checks: Vec<(usize, usize)>,
}

fn find(body: &Body, types: &TypeTable) -> Option<Plan> {
    let order = reverse_postorder(body);
    let headers = loop_headers(body, &order);
    let loops: BTreeMap<usize, HashSet<usize>> = natural_loops(body, &order, &headers).into_iter().collect();
    let mut preds: Vec<Vec<usize>> = vec![Vec::new(); body.blocks.len()];
    for &block in &order {
        for succ in crate::long_access_lint::successors(&body.blocks[block].terminator) {
            preds[succ].push(block);
        }
    }
    // A mutable borrow of a whole handle local could re-point it anywhere.
    let lent: HashSet<LocalId> = body
        .blocks
        .iter()
        .flat_map(|block| &block.stmts)
        .filter_map(|stmt| match &stmt.kind {
            StmtKind::Assign { rvalue: Rvalue::Ref { place, mutable: true }, .. } if place.projection.is_empty() => {
                Some(place.local)
            }
            _ => None,
        })
        .collect();
    for (&header, blocks) in &loops {
        if let Some(plan) = plan_for(body, types, header, blocks, &preds, &lent) {
            return Some(plan);
        }
    }
    None
}

fn plan_for(
    body: &Body,
    types: &TypeTable,
    header: usize,
    blocks: &HashSet<usize>,
    preds: &[Vec<usize>],
    lent: &HashSet<LocalId>,
) -> Option<Plan> {
    let entries: Vec<usize> = preds[header].iter().copied().filter(|p| !blocks.contains(p)).collect();
    let [entry] = entries.as_slice() else { return None };
    let head = &body.blocks[header];
    // The header only tests: pure statements into locals, then a two-way branch.
    if !head.stmts.iter().all(|stmt| pure(stmt)) {
        return None;
    }
    let Terminator::SwitchInt { discr: Operand::Copy(discr) | Operand::Move(discr), targets, otherwise } = &head.terminator
    else {
        return None;
    };
    let [(_, first)] = targets.as_slice() else { return None };
    let (first, otherwise) = (first.0 as usize, otherwise.0 as usize);
    let (inside, exit) = match (blocks.contains(&first), blocks.contains(&otherwise)) {
        (true, false) => (first, otherwise),
        (false, true) => (otherwise, first),
        _ => return None,
    };
    if !discr.projection.is_empty() {
        return None;
    }
    // Nothing in the loop begins or ends an access but a check, drops
    // anything, or runs Ember code.
    let mut written: HashSet<LocalId> = HashSet::new();
    for &block in blocks {
        let data = &body.blocks[block];
        let mut stmts = data.stmts.iter().peekable();
        while let Some(stmt) = stmts.next() {
            match &stmt.kind {
                StmtKind::BeginAccess { place, mutable } => {
                    let paired = stmts.next_if(|next| {
                        matches!(&next.kind, StmtKind::EndAccess { place: end, mutable: end_mutable }
                            if end == place && end_mutable == mutable)
                    });
                    paired?;
                }
                StmtKind::EndAccess { .. }
                | StmtKind::BeginAccessTransfer { .. }
                | StmtKind::EndAccessTransfer { .. }
                | StmtKind::Drop { .. } => return None,
                StmtKind::Assign { place, .. } => {
                    written.insert(place.local);
                }
                StmtKind::CheckedBinaryOp { dest, overflow, .. } => {
                    written.insert(dest.local);
                    written.insert(overflow.local);
                }
                StmtKind::StorageLive(_) | StmtKind::StorageDead(_) | StmtKind::Nop => {}
            }
        }
        if let Terminator::Call { func, args, dest, .. } = &data.terminator {
            let FuncRef::Builtin { which, .. } = func else { return None };
            if !runs_no_ember_code(body, types, which, args) {
                return None;
            }
            written.insert(dest.local);
        }
    }
    // The one path in each turn: from the header's inside edge through blocks
    // that only go on, their statements pure but for the checks.
    let mut checks = Vec::new();
    let mut current = inside;
    loop {
        if preds[current].len() != 1 || current == header {
            break;
        }
        let data = &body.blocks[current];
        let mut index = 0;
        let mut pure_so_far = true;
        while index < data.stmts.len() {
            match &data.stmts[index].kind {
                StmtKind::BeginAccess { place, .. } => {
                    let stable = !written.contains(&place.local)
                        && !lent.contains(&place.local)
                        && place.projection.iter().all(|p| matches!(p, ember_mir::Projection::Field(_)))
                        && matches!(body.local(place.local).kind, LocalKind::Arg | LocalKind::User | LocalKind::Temp);
                    if !stable {
                        pure_so_far = false;
                        break;
                    }
                    checks.push((current, index));
                    index += 2;
                }
                _ if pure(&data.stmts[index]) => index += 1,
                _ => {
                    pure_so_far = false;
                    break;
                }
            }
        }
        match &data.terminator {
            Terminator::Goto(next) if pure_so_far => current = next.0 as usize,
            _ => break,
        }
    }
    if checks.is_empty() {
        return None;
    }
    Some(Plan { header, entry: *entry, inside, exit, checks })
}

/// A statement anything outside the function could see nothing of: storage
/// markers, and a local set from values (no call, no check, no write through
/// memory).
fn pure(stmt: &Stmt) -> bool {
    match &stmt.kind {
        StmtKind::StorageLive(_) | StmtKind::StorageDead(_) | StmtKind::Nop => true,
        StmtKind::Assign { place, rvalue } => {
            place.projection.is_empty()
                && matches!(
                    rvalue,
                    Rvalue::Use(_) | Rvalue::BinaryOp { .. } | Rvalue::UnaryOp { .. } | Rvalue::Cast { .. }
                )
        }
        _ => false,
    }
}

/// A built-in that begins no access and runs no Ember code: list and text
/// growth and reads, and printing text or numbers. Removing or clearing
/// elements drops them, which runs code unless they are plain values.
fn runs_no_ember_code(body: &Body, types: &TypeTable, which: &Builtin, args: &[Operand]) -> bool {
    match which {
        Builtin::ArrayPush
        | Builtin::ArrayLen
        | Builtin::ArrayCapacity
        | Builtin::ArrayReserve
        | Builtin::SpanLen
        | Builtin::StringLen
        | Builtin::StringPush
        | Builtin::StringPushChar
        | Builtin::StringInsert
        | Builtin::StringRemove
        | Builtin::StringTruncate
        | Builtin::Print
        | Builtin::Println => true,
        Builtin::ArrayClear | Builtin::ArrayTruncate => args.first().is_some_and(|list| {
            let (Operand::Copy(place) | Operand::Move(place)) = list else { return false };
            let mut ty = crate::regions::place_type(body, types, place);
            if let TyKind::Ref { inner, .. } = *types.kind(ty) {
                ty = inner;
            }
            matches!(*types.kind(ty), TyKind::Vec { elem, .. }
                if matches!(types.kind(elem), TyKind::Int(_) | TyKind::Uint(_) | TyKind::Float(_) | TyKind::Bool | TyKind::Char))
        }),
        _ => false,
    }
}

fn apply(body: &mut Body, plan: Plan) {
    let head = body.blocks[plan.header].clone();
    // The guard computes the loop's test again into fresh locals, so the loop
    // keeps its own test to itself.
    let mut fresh: BTreeMap<LocalId, LocalId> = BTreeMap::new();
    let mut guard_stmts = Vec::new();
    for stmt in &head.stmts {
        let StmtKind::Assign { place, rvalue } = &stmt.kind else { continue };
        let mut rvalue = rvalue.clone();
        rename_operands(&mut rvalue, &fresh);
        // The header reads these again: the guard only copies them.
        for_each_operand(&mut rvalue, &mut |operand| {
            if let Operand::Move(place) = operand {
                *operand = Operand::Copy(place.clone());
            }
        });
        let decl = body.locals[place.local.0 as usize].clone();
        let local = LocalId(body.locals.len() as u32);
        body.locals.push(ember_mir::LocalDecl { kind: LocalKind::Temp, name: None, ..decl });
        fresh.insert(place.local, local);
        guard_stmts.push(Stmt { kind: StmtKind::Assign { place: Place::local(local), rvalue }, span: stmt.span });
    }
    let Terminator::SwitchInt { discr, targets, otherwise } = &head.terminator else {
        unreachable!("a planned header ends in a switch")
    };
    let mut discr = discr.clone();
    rename_operand(&mut discr, &fresh);
    let checks_block = body.blocks.len() + 1;
    let redirect = |target: BasicBlockId| {
        if target.0 as usize == plan.inside { BasicBlockId(checks_block as u32) } else { BasicBlockId(plan.exit as u32) }
    };
    let guard = BasicBlock {
        stmts: guard_stmts,
        terminator: Terminator::SwitchInt {
            discr,
            targets: targets.iter().map(|(value, target)| (*value, redirect(*target))).collect(),
            otherwise: redirect(*otherwise),
        },
        terminator_span: head.terminator_span,
    };
    let mut check_stmts = Vec::new();
    for &(block, index) in &plan.checks {
        let begin = body.blocks[block].stmts[index].clone();
        let end = body.blocks[block].stmts[index + 1].clone();
        let StmtKind::BeginAccess { place, mutable } = &begin.kind else { unreachable!("a planned check begins") };
        body.hoisted_accesses.push(HoistedAccess {
            span: begin.span,
            loop_span: head.terminator_span,
            preheader: BasicBlockId(checks_block as u32),
            preheader_statement: check_stmts.len(),
            postheader: BasicBlockId(checks_block as u32),
            place: place.clone(),
            mutable: *mutable,
            proof: HoistedAccessProof::NoAccessInLoop,
        });
        check_stmts.push(begin);
        check_stmts.push(end);
        body.blocks[block].stmts[index].kind = StmtKind::Nop;
        body.blocks[block].stmts[index + 1].kind = StmtKind::Nop;
    }
    let checks = BasicBlock {
        stmts: check_stmts,
        terminator: Terminator::Goto(BasicBlockId(plan.header as u32)),
        terminator_span: head.terminator_span,
    };
    let guard_block = body.blocks.len();
    body.blocks.push(guard);
    body.blocks.push(checks);
    retarget(&mut body.blocks[plan.entry].terminator, plan.header, guard_block);
}

fn retarget(terminator: &mut Terminator, from: usize, to: usize) {
    let swap = |target: &mut BasicBlockId| {
        if target.0 as usize == from {
            *target = BasicBlockId(to as u32);
        }
    };
    match terminator {
        Terminator::Goto(target) | Terminator::Call { next: target, .. } | Terminator::Assert { next: target, .. } => {
            swap(target)
        }
        Terminator::SwitchInt { targets, otherwise, .. } => {
            targets.iter_mut().for_each(|(_, target)| swap(target));
            swap(otherwise);
        }
        Terminator::Return | Terminator::Unreachable => {}
    }
}

fn rename_operand(operand: &mut Operand, fresh: &BTreeMap<LocalId, LocalId>) {
    if let Operand::Copy(place) | Operand::Move(place) = operand
        && place.projection.is_empty()
        && let Some(&local) = fresh.get(&place.local)
    {
        *operand = Operand::Copy(Place::local(local));
    }
}

fn rename_operands(rvalue: &mut Rvalue, fresh: &BTreeMap<LocalId, LocalId>) {
    for_each_operand(rvalue, &mut |operand| rename_operand(operand, fresh));
}

fn for_each_operand(rvalue: &mut Rvalue, f: &mut impl FnMut(&mut Operand)) {
    match rvalue {
        Rvalue::Use(operand) | Rvalue::UnaryOp { operand, .. } | Rvalue::Cast { operand, .. } => f(operand),
        Rvalue::BinaryOp { lhs, rhs, .. } => {
            f(lhs);
            f(rhs);
        }
        _ => {}
    }
}
