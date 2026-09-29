//! Copy propagation: a value the compiler holds in a local of its own, set
//! once as a plain copy of another local, is read from that local wherever
//! the local still holds it.
//!
//! The lowering holds a value it must evaluate once (`enumerate`'s `start`,
//! a count) in a hidden local. Where that local is only a copy of another,
//! the copy stands in an outer loop's turn before its inner loop, and a nest
//! with work between its loops is not perfect: MSVC reorders only a perfect
//! nest (`enumerate(start=round)`: 1.5x C, and C's speed without the copy).
//! Locals the programmer named keep their reads, for the debugger.

use ember_mir::{Body, LocalId, LocalKind, Operand, Place, Rvalue, StmtKind, Terminator};
use ember_types::{TyKind, TypeTable};

pub fn propagate_copies_all(bodies: &mut [Body], types: &TypeTable) -> usize {
    bodies.iter_mut().map(|body| propagate_copies(body, types)).sum()
}

fn propagate_copies(body: &mut Body, types: &TypeTable) -> usize {
    let mut total = 0;
    // A copy of a copy reads the first source after a second round.
    for _ in 0..4 {
        let mut replaced = 0;
        for (copy, defs) in candidates(body, types) {
            replaced += propagate(body, copy, &defs);
        }
        total += replaced;
        if replaced == 0 {
            break;
        }
    }
    total
}

fn scalar(types: &TypeTable, body: &Body, local: LocalId) -> bool {
    matches!(
        types.kind(body.local(local).ty),
        TyKind::Int(_) | TyKind::Uint(_) | TyKind::Bool | TyKind::Char | TyKind::Float(_) | TyKind::Range(_)
    )
}

/// The hidden scalar locals given only copies of one local, `copy = source`
/// (once, or on each copy of a loop that versioning made), with a scalar
/// source of the same type whose address is never taken, and named nowhere
/// but as a value read; each with where it is set.
fn candidates(body: &Body, types: &TypeTable) -> Vec<(LocalId, Vec<(usize, usize)>)> {
    let n = body.locals.len();
    let mut source_of: Vec<Option<LocalId>> = vec![None; n];
    let mut defs: Vec<Vec<(usize, usize)>> = vec![Vec::new(); n];
    let mut other = vec![false; n];
    let mut taken = vec![false; n];
    for (block, data) in body.blocks.iter().enumerate() {
        for (index, stmt) in data.stmts.iter().enumerate() {
            match &stmt.kind {
                StmtKind::Assign { place, rvalue } => {
                    let local = place.local.0 as usize;
                    match rvalue {
                        Rvalue::Use(Operand::Copy(source) | Operand::Move(source))
                            if place.projection.is_empty()
                                && source.projection.is_empty()
                                && source_of[local].is_none_or(|seen| seen == source.local) =>
                        {
                            source_of[local] = Some(source.local);
                            defs[local].push((block, index));
                        }
                        _ => other[local] = true,
                    }
                    if let Rvalue::Ref { place, .. } | Rvalue::Discriminant(place) = rvalue {
                        taken[place.local.0 as usize] = true;
                    }
                }
                StmtKind::CheckedBinaryOp { dest, overflow, .. } => {
                    other[dest.local.0 as usize] = true;
                    other[overflow.local.0 as usize] = true;
                }
                StmtKind::Drop { place, .. }
                | StmtKind::BeginAccess { place, .. }
                | StmtKind::BeginAccessTransfer { place, .. }
                | StmtKind::EndAccess { place, .. }
                | StmtKind::EndAccessTransfer { place, .. } => other[place.local.0 as usize] = true,
                StmtKind::StorageLive(_) | StmtKind::StorageDead(_) | StmtKind::Nop => {}
            }
        }
        if let Terminator::Call { dest, .. } = &data.terminator {
            other[dest.local.0 as usize] = true;
        }
    }
    (0..n)
        .filter_map(|local| {
            let copy = LocalId(local as u32);
            let decl = body.local(copy);
            let hidden = matches!(decl.kind, LocalKind::Temp)
                || (matches!(decl.kind, LocalKind::User) && decl.name.as_deref().is_none_or(str::is_empty));
            let source = source_of[local]?;
            let fits = hidden
                && !other[local]
                && !taken[local]
                && source != copy
                && matches!(body.local(source).kind, LocalKind::User | LocalKind::Temp | LocalKind::Arg)
                && !taken[source.0 as usize]
                && body.local(source).ty == decl.ty
                && scalar(types, body, copy);
            fits.then(|| (copy, defs[local].clone()))
        })
        .collect()
}

/// Replaces the reads of `copy`, set by the statements `defs` (block,
/// index), by its source where the source still holds the value copied;
/// removes the copies once nothing reads it. Returns how many reads changed.
fn propagate(body: &mut Body, copy: LocalId, defs: &[(usize, usize)]) -> usize {
    // An earlier copy's reads, replaced this round, may have changed a
    // source: every statement must still copy the same one.
    let source_at = |(block, index): (usize, usize)| match &body.blocks[block].stmts[index].kind {
        StmtKind::Assign { rvalue: Rvalue::Use(Operand::Copy(source) | Operand::Move(source)), .. } => Some(source.local),
        _ => None,
    };
    let Some(source) = defs.first().and_then(|&def| source_at(def)) else { return 0 };
    if source == copy || defs.iter().any(|&def| source_at(def) != Some(source)) {
        return 0;
    }
    let is_def = |block: usize, index: usize| defs.contains(&(block, index));

    // Where `copy == source` holds: after the copy, until the source is
    // written or its storage begins or ends. A must-analysis: every path in.
    let n = body.blocks.len();
    let mut predecessors = vec![Vec::new(); n];
    for (block, data) in body.blocks.iter().enumerate() {
        for next in successors(&data.terminator) {
            predecessors[next].push(block);
        }
    }
    let kills = |kind: &StmtKind| match kind {
        StmtKind::Assign { place, .. } => place.local == source,
        StmtKind::CheckedBinaryOp { dest, overflow, .. } => dest.local == source || overflow.local == source,
        StmtKind::Drop { place, .. } => place.local == source,
        StmtKind::StorageLive(local) | StmtKind::StorageDead(local) => *local == source,
        _ => false,
    };
    let out_of = |block: usize, mut holds: bool| {
        for (index, stmt) in body.blocks[block].stmts.iter().enumerate() {
            if is_def(block, index) {
                holds = true;
            } else if kills(&stmt.kind) {
                holds = false;
            }
        }
        if let Terminator::Call { dest, .. } = &body.blocks[block].terminator
            && dest.local == source
        {
            holds = false;
        }
        holds
    };
    let mut holds_in = vec![true; n];
    holds_in[0] = false;
    for (block, from) in predecessors.iter().enumerate() {
        if from.is_empty() {
            holds_in[block] = false;
        }
    }
    let mut holds_out: Vec<bool> = (0..n).map(|block| out_of(block, holds_in[block])).collect();
    loop {
        let mut changed = false;
        for block in 1..n {
            let holds = !predecessors[block].is_empty() && predecessors[block].iter().all(|&from| holds_out[from]);
            if holds != holds_in[block] {
                holds_in[block] = holds;
                holds_out[block] = out_of(block, holds);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }

    let mut replaced = 0;
    let swap = |operand: &mut Operand, replaced: &mut usize| {
        if let Operand::Copy(place) | Operand::Move(place) = operand
            && place.local == copy
        {
            *operand = Operand::Copy(Place::local(source));
            *replaced += 1;
        }
    };
    for block in 0..n {
        let mut holds = holds_in[block];
        let data = &mut body.blocks[block];
        for (index, stmt) in data.stmts.iter_mut().enumerate() {
            // A statement reads before it writes.
            if holds && let StmtKind::Assign { rvalue, .. } = &mut stmt.kind {
                match rvalue {
                    Rvalue::Use(value) | Rvalue::Cast { operand: value, .. } | Rvalue::UnaryOp { operand: value, .. } => {
                        swap(value, &mut replaced)
                    }
                    Rvalue::BinaryOp { lhs, rhs, .. } => {
                        swap(lhs, &mut replaced);
                        swap(rhs, &mut replaced);
                    }
                    Rvalue::Aggregate { operands, .. } => operands.iter_mut().for_each(|value| swap(value, &mut replaced)),
                    Rvalue::Repeat { value, .. } => swap(value, &mut replaced),
                    Rvalue::Ref { .. } | Rvalue::Discriminant(_) => {}
                }
            }
            if holds && let StmtKind::CheckedBinaryOp { lhs, rhs, .. } = &mut stmt.kind {
                swap(lhs, &mut replaced);
                swap(rhs, &mut replaced);
            }
            if is_def(block, index) {
                holds = true;
            } else if kills(&stmt.kind) {
                holds = false;
            }
        }
        if holds {
            match &mut data.terminator {
                Terminator::SwitchInt { discr, .. } => swap(discr, &mut replaced),
                Terminator::Call { args, .. } => args.iter_mut().for_each(|arg| swap(arg, &mut replaced)),
                Terminator::Assert { cond, msg, .. } => {
                    swap(cond, &mut replaced);
                    if let ember_mir::AssertKind::Bounds { len, index } = msg {
                        swap(len, &mut replaced);
                        swap(index, &mut replaced);
                    }
                }
                Terminator::Goto(_) | Terminator::Return | Terminator::Unreachable => {}
            }
        }
    }
    if replaced > 0 && !read_anywhere(body, copy) {
        for &(block, index) in defs {
            body.blocks[block].stmts[index].kind = StmtKind::Nop;
        }
    }
    replaced
}

fn read_anywhere(body: &Body, local: LocalId) -> bool {
    let mut read = vec![false; body.locals.len()];
    for data in &body.blocks {
        data.stmts.iter().for_each(|stmt| crate::strength_reduce::stmt_reads(stmt, &mut read));
        crate::strength_reduce::terminator_reads(&data.terminator, &mut read);
    }
    read[local.0 as usize]
}

fn successors(terminator: &Terminator) -> Vec<usize> {
    match terminator {
        Terminator::Goto(next) | Terminator::Call { next, .. } | Terminator::Assert { next, .. } => vec![next.0 as usize],
        Terminator::SwitchInt { targets, otherwise, .. } => {
            targets.iter().map(|(_, target)| target.0 as usize).chain([otherwise.0 as usize]).collect()
        }
        Terminator::Return | Terminator::Unreachable => Vec::new(),
    }
}
