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

use std::collections::HashMap;

use ember_mir::{Body, CastKind, LocalId, LocalKind, Operand, Place, Rvalue, StmtKind, Terminator};
use ember_types::{Ty, TyKind, TypeTable};

pub fn propagate_copies_all(bodies: &mut [Body], types: &TypeTable) -> usize {
    bodies.iter_mut().map(|body| propagate_copies(body, types)).sum()
}

fn propagate_copies(body: &mut Body, types: &TypeTable) -> usize {
    let mut total = 0;
    // A copy of a copy reads the first source after a second round.
    for _ in 0..4 {
        let replaced = propagate(body, &candidates(body, types));
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
                | StmtKind::EndAccessTransfer { place, .. } | StmtKind::LendAccess { place, .. } => other[place.local.0 as usize] = true,
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

/// Replaces the reads of each copy, set by its statements (block, index), by
/// its source where the source still holds the value copied; removes a
/// copy's statements once nothing reads it. Returns how many reads changed.
/// One analysis for every copy, a fact per copy ("the copy equals its
/// source") in bit sets, so the pass grows with the function, not with the
/// function times its copies (D-811).
fn propagate(body: &mut Body, copies: &[(LocalId, Vec<(usize, usize)>)]) -> usize {
    // A copy whose statements no longer all copy one source (an earlier
    // round's replacement changed one) waits for the next round.
    let source_at = |body: &Body, (block, index): (usize, usize)| match &body.blocks[block].stmts[index].kind {
        StmtKind::Assign { rvalue: Rvalue::Use(Operand::Copy(source) | Operand::Move(source)), .. } => Some(source.local),
        _ => None,
    };
    let facts: Vec<(LocalId, LocalId, &[(usize, usize)])> = copies
        .iter()
        .filter_map(|(copy, defs)| {
            let source = source_at(body, *defs.first()?)?;
            (source != *copy && defs.iter().all(|&def| source_at(body, def) == Some(source)))
                .then_some((*copy, source, defs.as_slice()))
        })
        .collect();
    if facts.is_empty() {
        return 0;
    }
    let words = facts.len().div_ceil(64);
    // Each copy's fact; the facts a write of each local ends (those whose
    // source it is); the fact each copying statement begins.
    let mut fact_of = vec![None; body.locals.len()];
    let mut ended_by: Vec<Vec<usize>> = vec![Vec::new(); body.locals.len()];
    let mut begun_at: HashMap<(usize, usize), usize> = HashMap::new();
    for (fact, &(copy, source, defs)) in facts.iter().enumerate() {
        fact_of[copy.0 as usize] = Some(fact);
        ended_by[source.0 as usize].push(fact);
        for &def in defs {
            begun_at.insert(def, fact);
        }
    }
    let written = |kind: &StmtKind| -> Vec<LocalId> {
        match kind {
            StmtKind::Assign { place, .. } | StmtKind::Drop { place, .. } => vec![place.local],
            StmtKind::CheckedBinaryOp { dest, overflow, .. } => vec![dest.local, overflow.local],
            StmtKind::StorageLive(local) | StmtKind::StorageDead(local) => vec![*local],
            _ => Vec::new(),
        }
    };

    // Each block's effect: the facts it begins and those it ends.
    let n = body.blocks.len();
    let mut begins = vec![vec![0u64; words]; n];
    let mut ends = vec![vec![0u64; words]; n];
    let mut predecessors = vec![Vec::new(); n];
    for (block, data) in body.blocks.iter().enumerate() {
        for next in successors(&data.terminator) {
            predecessors[next].push(block);
        }
        let end_by = |local: LocalId, begins: &mut [u64], ends: &mut [u64]| {
            for &fact in &ended_by[local.0 as usize] {
                clear(begins, fact);
                mark(ends, fact);
            }
        };
        for (index, stmt) in data.stmts.iter().enumerate() {
            for local in written(&stmt.kind) {
                end_by(local, &mut begins[block], &mut ends[block]);
            }
            if let Some(&fact) = begun_at.get(&(block, index)) {
                mark(&mut begins[block], fact);
                clear(&mut ends[block], fact);
            }
        }
        if let Terminator::Call { dest, .. } = &data.terminator {
            end_by(dest.local, &mut begins[block], &mut ends[block]);
        }
    }
    // A must-analysis, every path in: optimistic everywhere but the entry and
    // the blocks nothing reaches, then down to the fixed point.
    let mut holds_in = vec![vec![!0u64; words]; n];
    for (block, from) in predecessors.iter().enumerate() {
        if block == 0 || from.is_empty() {
            holds_in[block] = vec![0; words];
        }
    }
    let out_of = |block: usize, entry: &[u64]| -> Vec<u64> {
        (0..words).map(|word| (entry[word] & !ends[block][word]) | begins[block][word]).collect()
    };
    let mut holds_out: Vec<Vec<u64>> = (0..n).map(|block| out_of(block, &holds_in[block])).collect();
    loop {
        let mut changed = false;
        for block in 1..n {
            if predecessors[block].is_empty() {
                continue;
            }
            let entry: Vec<u64> = (0..words)
                .map(|word| predecessors[block].iter().fold(!0u64, |all, &from| all & holds_out[from][word]))
                .collect();
            if entry != holds_in[block] {
                holds_out[block] = out_of(block, &entry);
                holds_in[block] = entry;
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }

    let mut replaced = 0;
    let mut changed = vec![false; facts.len()];
    let mut swap = |operand: &mut Operand, set: &[u64], replaced: &mut usize| {
        if let Operand::Copy(place) | Operand::Move(place) = operand
            && let Some(fact) = fact_of[place.local.0 as usize]
            && holds(set, fact)
        {
            *operand = Operand::Copy(Place::local(facts[fact].1));
            *replaced += 1;
            changed[fact] = true;
        }
    };
    for (block, entry) in holds_in.iter().enumerate() {
        let mut set = entry.clone();
        let data = &mut body.blocks[block];
        for (index, stmt) in data.stmts.iter_mut().enumerate() {
            // A statement reads before it writes.
            match &mut stmt.kind {
                StmtKind::Assign { rvalue, .. } => match rvalue {
                    Rvalue::Use(value) | Rvalue::Cast { operand: value, .. } | Rvalue::UnaryOp { operand: value, .. } => {
                        swap(value, &set, &mut replaced)
                    }
                    Rvalue::BinaryOp { lhs, rhs, .. } => {
                        swap(lhs, &set, &mut replaced);
                        swap(rhs, &set, &mut replaced);
                    }
                    Rvalue::Aggregate { operands, .. } => operands.iter_mut().for_each(|value| swap(value, &set, &mut replaced)),
                    Rvalue::Repeat { value, .. } => swap(value, &set, &mut replaced),
                    Rvalue::Ref { .. } | Rvalue::Discriminant(_) => {}
                },
                StmtKind::CheckedBinaryOp { lhs, rhs, .. } => {
                    swap(lhs, &set, &mut replaced);
                    swap(rhs, &set, &mut replaced);
                }
                _ => {}
            }
            for local in written(&stmt.kind) {
                for &fact in &ended_by[local.0 as usize] {
                    clear(&mut set, fact);
                }
            }
            if let Some(&fact) = begun_at.get(&(block, index)) {
                mark(&mut set, fact);
            }
        }
        match &mut data.terminator {
            Terminator::SwitchInt { discr, .. } => swap(discr, &set, &mut replaced),
            Terminator::Call { args, .. } => args.iter_mut().for_each(|arg| swap(arg, &set, &mut replaced)),
            Terminator::Assert { cond, msg, .. } => {
                swap(cond, &set, &mut replaced);
                if let ember_mir::AssertKind::Bounds { len, index } = msg {
                    swap(len, &set, &mut replaced);
                    swap(index, &set, &mut replaced);
                }
            }
            Terminator::Goto(_) | Terminator::Return | Terminator::Unreachable => {}
        }
    }
    if replaced > 0 {
        let read = reads(body);
        for (&(copy, _, defs), _) in facts.iter().zip(&changed).filter(|(_, changed)| **changed) {
            if !read[copy.0 as usize] {
                for &(block, index) in defs {
                    body.blocks[block].stmts[index].kind = StmtKind::Nop;
                }
            }
        }
    }
    replaced
}

fn mark(set: &mut [u64], fact: usize) {
    set[fact / 64] |= 1 << (fact % 64);
}

fn clear(set: &mut [u64], fact: usize) {
    set[fact / 64] &= !(1 << (fact % 64));
}

fn holds(set: &[u64], fact: usize) -> bool {
    set[fact / 64] & (1 << (fact % 64)) != 0
}

/// G8-4 decision C — an integer widened and read back no wider than it was:
/// `wide = a as T`, `T` wider than `a`'s type `S`, then `c = wide as U` with
/// `U` no wider than `S`, reads `a` (`c = a as U`, or `c = a` when `U` is
/// `S`) wherever `wide` still holds that widening of `a`: the low bits are
/// `a`'s either way. The copy of a loop that computes `enumerate`'s wide
/// numbers as `int`s widens each one, and a program reading it back as an
/// `int` then reads the `int`: MSVC kept the 128-bit halves in memory on every
/// turn (1.47x the loop with `int` numbers, and its speed without them).
/// Returns how many reads changed.
pub fn narrow_widened_reads_all(bodies: &mut [Body], types: &TypeTable) -> usize {
    bodies.iter_mut().map(|body| narrow_widened_reads(body, types)).sum()
}

fn integer_width(types: &TypeTable, ty: Ty) -> Option<u64> {
    matches!(types.kind(ty), TyKind::Int(_) | TyKind::Uint(_)).then(|| ember_types::bit_width(types, ty)).flatten()
}

fn narrow_widened_reads(body: &mut Body, types: &TypeTable) -> usize {
    // Each (wide, a) pair with the statements that widen `a` into `wide`.
    let mut taken = vec![false; body.locals.len()];
    let mut widenings: Vec<((LocalId, LocalId), Vec<(usize, usize)>)> = Vec::new();
    for (block, data) in body.blocks.iter().enumerate() {
        for (index, stmt) in data.stmts.iter().enumerate() {
            let StmtKind::Assign { place, rvalue } = &stmt.kind else { continue };
            if let Rvalue::Ref { place, .. } | Rvalue::Discriminant(place) = rvalue {
                taken[place.local.0 as usize] = true;
            }
            let Rvalue::Cast { kind: CastKind::Numeric, operand: Operand::Copy(source), to } = rvalue else { continue };
            if !place.projection.is_empty() || !source.projection.is_empty() || place.local == source.local {
                continue;
            }
            let (Some(from), Some(wide)) = (integer_width(types, body.local(source.local).ty), integer_width(types, *to)) else {
                continue;
            };
            if wide <= from || body.local(place.local).ty != *to {
                continue;
            }
            let key = (place.local, source.local);
            match widenings.iter_mut().find(|(seen, _)| *seen == key) {
                Some((_, defs)) => defs.push((block, index)),
                None => widenings.push((key, vec![(block, index)])),
            }
        }
    }
    let mut total = 0;
    for ((wide, source), defs) in widenings {
        if taken[wide.0 as usize] || taken[source.0 as usize] {
            continue;
        }
        total += narrow_reads_of(body, types, wide, source, &defs);
    }
    total
}

/// `narrow_widened_reads` for one `wide = source as T`, set by `defs`.
fn narrow_reads_of(body: &mut Body, types: &TypeTable, wide: LocalId, source: LocalId, defs: &[(usize, usize)]) -> usize {
    let source_ty = body.local(source).ty;
    let Some(width) = integer_width(types, source_ty) else { return 0 };
    let is_def = |block: usize, index: usize| defs.contains(&(block, index));
    // Where `wide == source as T` holds: after a widening, until either is
    // written or its storage begins or ends. A must-analysis: every path in.
    let n = body.blocks.len();
    let mut predecessors = vec![Vec::new(); n];
    for (block, data) in body.blocks.iter().enumerate() {
        for next in successors(&data.terminator) {
            predecessors[next].push(block);
        }
    }
    let touches = |local: LocalId| local == wide || local == source;
    let kills = |kind: &StmtKind| match kind {
        StmtKind::Assign { place, .. } => touches(place.local),
        StmtKind::CheckedBinaryOp { dest, overflow, .. } => touches(dest.local) || touches(overflow.local),
        StmtKind::Drop { place, .. } => touches(place.local),
        StmtKind::StorageLive(local) | StmtKind::StorageDead(local) => touches(*local),
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
            && touches(dest.local)
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
    for (block, holds_at_entry) in holds_in.iter().enumerate() {
        let mut holds = *holds_at_entry;
        for index in 0..body.blocks[block].stmts.len() {
            if holds
                && let StmtKind::Assign { rvalue, .. } = &mut body.blocks[block].stmts[index].kind
                && let Rvalue::Cast { kind: CastKind::Numeric, operand: Operand::Copy(read), to } = rvalue
                && *read == Place::local(wide)
                && integer_width(types, *to).is_some_and(|narrow| narrow <= width)
            {
                *rvalue = if *to == source_ty {
                    Rvalue::Use(Operand::Copy(Place::local(source)))
                } else {
                    Rvalue::Cast { kind: CastKind::Numeric, operand: Operand::Copy(Place::local(source)), to: *to }
                };
                replaced += 1;
            }
            let kind = &body.blocks[block].stmts[index].kind;
            if is_def(block, index) {
                holds = true;
            } else if kills(kind) {
                holds = false;
            }
        }
    }
    // A local the programmer named keeps its value, for the debugger.
    let decl = body.local(wide);
    let hidden = matches!(decl.kind, LocalKind::Temp)
        || (matches!(decl.kind, LocalKind::User) && decl.name.as_deref().is_none_or(str::is_empty));
    if replaced > 0 && hidden && !read_anywhere(body, wide) {
        for &(block, index) in defs {
            body.blocks[block].stmts[index].kind = StmtKind::Nop;
        }
    }
    replaced
}

fn read_anywhere(body: &Body, local: LocalId) -> bool {
    reads(body)[local.0 as usize]
}

/// Which locals anything reads.
fn reads(body: &Body) -> Vec<bool> {
    let mut read = vec![false; body.locals.len()];
    for data in &body.blocks {
        data.stmts.iter().for_each(|stmt| crate::strength_reduce::stmt_reads(stmt, &mut read));
        crate::strength_reduce::terminator_reads(&data.terminator, &mut read);
    }
    read
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
