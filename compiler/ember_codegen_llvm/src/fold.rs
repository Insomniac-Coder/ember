//! `[GRM-39]` (G7-2) — a literal's temporary that the next literal reads once is written straight
//! into that literal's field, never given a slot of its own: a value nested 256 deep was 256
//! temporaries of growing size in one frame, and four of them overflowed a 1 MB stack in the debug
//! profile. The same choice as the C backend's `foldable_aggregates`, made from the same MIR.

use std::collections::{BTreeMap, HashMap};

use ember_mir::{AggregateKind, AssertKind, Body, FuncRef, LocalId, LocalKind, Operand, Place, Projection, Rvalue, Stmt, StmtKind, Terminator};
use ember_types::{TyKind, TypeTable};

fn place_locals(place: &Place, out: &mut Vec<LocalId>) {
    out.push(place.local);
    for projection in &place.projection {
        if let Projection::Index(local) = projection {
            out.push(*local);
        }
    }
}

fn operand_locals(operand: &Operand, out: &mut Vec<LocalId>) {
    if let Operand::Copy(place) | Operand::Move(place) = operand {
        place_locals(place, out);
    }
}

fn rvalue_locals(rvalue: &Rvalue, out: &mut Vec<LocalId>) {
    match rvalue {
        Rvalue::Use(o) | Rvalue::UnaryOp { operand: o, .. } | Rvalue::Cast { operand: o, .. } => operand_locals(o, out),
        Rvalue::BinaryOp { lhs, rhs, .. } => {
            operand_locals(lhs, out);
            operand_locals(rhs, out);
        }
        Rvalue::Aggregate { operands, .. } => operands.iter().for_each(|o| operand_locals(o, out)),
        Rvalue::Repeat { value, .. } => operand_locals(value, out),
        Rvalue::Discriminant(place) | Rvalue::Ref { place, .. } => place_locals(place, out),
    }
}

fn stmt_locals(stmt: &Stmt, out: &mut Vec<LocalId>) {
    match &stmt.kind {
        StmtKind::Assign { place, rvalue } => {
            place_locals(place, out);
            rvalue_locals(rvalue, out);
        }
        StmtKind::CheckedBinaryOp { dest, overflow, lhs, rhs, .. } => {
            place_locals(dest, out);
            place_locals(overflow, out);
            operand_locals(lhs, out);
            operand_locals(rhs, out);
        }
        StmtKind::Drop { place, .. }
        | StmtKind::BeginAccess { place, .. }
        | StmtKind::BeginAccessTransfer { place, .. }
        | StmtKind::EndAccess { place, .. }
        | StmtKind::EndAccessTransfer { place, .. }
        | StmtKind::LendAccess { place, .. } => place_locals(place, out),
        StmtKind::StorageLive(_) | StmtKind::StorageDead(_) | StmtKind::Nop => {}
    }
}

fn terminator_locals(terminator: &Terminator, out: &mut Vec<LocalId>) {
    match terminator {
        Terminator::SwitchInt { discr, .. } => operand_locals(discr, out),
        Terminator::Call { args, dest, func, .. } => {
            args.iter().for_each(|a| operand_locals(a, out));
            place_locals(dest, out);
            if let FuncRef::Indirect { operand, .. } = func {
                operand_locals(operand, out);
            }
        }
        Terminator::Assert { cond, msg, .. } => {
            operand_locals(cond, out);
            match msg {
                AssertKind::Bounds { len, index } => {
                    operand_locals(len, out);
                    operand_locals(index, out);
                }
                AssertKind::RefCellBorrow { file, line } => {
                    operand_locals(file, out);
                    operand_locals(line, out);
                }
                AssertKind::Panic { message } => operand_locals(message, out),
                _ => {}
            }
        }
        Terminator::Goto(_) | Terminator::Return | Terminator::Unreachable => {}
    }
}

/// D-569 — the functions whose views (`str`, `Span`, `MutSpan`) go as two words, a pointer and a
/// length, where Windows x64 passes a 16-byte view behind a pointer to a copy: the C backend's
/// `split_view_params`, the same bodies. Only a function called by name alone qualifies (no value
/// of it is taken, it fills no table slot), so every call is this module's own.
pub(crate) fn split_view_functions(bodies: &[Body]) -> std::collections::BTreeSet<String> {
    let mut taken: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
    fn mentions<'b>(operand: &'b Operand, taken: &mut std::collections::BTreeSet<&'b str>) {
        if let Operand::Const(ember_mir::Const::Fn(symbol)) = operand {
            taken.insert(symbol.as_str());
        }
    }
    for body in bodies {
        for block in &body.blocks {
            for stmt in &block.stmts {
                match &stmt.kind {
                    StmtKind::Assign { rvalue, .. } => match rvalue {
                        Rvalue::Use(o) | Rvalue::UnaryOp { operand: o, .. } | Rvalue::Repeat { value: o, .. } => mentions(o, &mut taken),
                        Rvalue::Cast { operand, kind, .. } => {
                            mentions(operand, &mut taken);
                            if let ember_mir::CastKind::InterfaceUpcast { implementations, .. }
                            | ember_mir::CastKind::ClassInterfaceUpcast { implementations, .. } = kind
                            {
                                taken.extend(implementations.iter().flatten().map(|method| method.symbol.as_str()));
                            }
                        }
                        Rvalue::BinaryOp { lhs, rhs, .. } => {
                            mentions(lhs, &mut taken);
                            mentions(rhs, &mut taken);
                        }
                        Rvalue::Aggregate { operands, .. } => operands.iter().for_each(|o| mentions(o, &mut taken)),
                        Rvalue::Ref { .. } | Rvalue::Discriminant(_) => {}
                    },
                    StmtKind::CheckedBinaryOp { lhs, rhs, .. } => {
                        mentions(lhs, &mut taken);
                        mentions(rhs, &mut taken);
                    }
                    _ => {}
                }
            }
            match &block.terminator {
                Terminator::Call { func, args, .. } => {
                    if let FuncRef::Indirect { operand, .. } = func {
                        mentions(operand, &mut taken);
                    }
                    args.iter().for_each(|o| mentions(o, &mut taken));
                }
                Terminator::SwitchInt { discr, .. } => mentions(discr, &mut taken),
                Terminator::Assert { cond, .. } => mentions(cond, &mut taken),
                _ => {}
            }
        }
    }
    bodies
        .iter()
        .filter(|body| {
            body.abi.is_none()
                && !body.is_extern_declaration
                && !body.is_abstract
                && !body.restrict_views
                && !body.is_lambda
                && body.closure_environment.is_none()
                && body.class_virtual_slot.is_none()
                && body.ffi_counted.is_none()
                && body.name != "main"
                && !taken.contains(body.symbol.as_str())
        })
        .map(|body| body.symbol.clone())
        .collect()
}

fn quiet(stmt: &Stmt) -> bool {
    matches!(stmt.kind, StmtKind::StorageLive(_) | StmtKind::StorageDead(_) | StmtKind::Nop)
}

/// Whether a literal of `rvalue` into a local of type `ty` is written one field at a time: a
/// struct (not a box or a `Shared`), a tuple, or a payload enum with a tag.
fn fields_writable(types: &TypeTable, ty: ember_types::Ty, rvalue: &Rvalue) -> bool {
    let Rvalue::Aggregate { kind, operands } = rvalue else { return false };
    if operands.is_empty() {
        return false;
    }
    match (kind, types.kind(ty)) {
        (AggregateKind::Struct(id), TyKind::Struct(local)) => {
            id == local && types.compiler_box_inner(*id).is_none() && crate::func::shared_inner(types, *id).is_none()
        }
        (AggregateKind::Tuple, TyKind::Tuple(_)) => true,
        (AggregateKind::Enum(id, _), TyKind::Enum(local)) => id == local && !types.enum_is_bare(*id) && types.option_niche(*id).is_none(),
        _ => false,
    }
}

/// Whether a literal's operands, or those of a literal folded into it, read `local`.
fn reads_local(folded: &BTreeMap<LocalId, Rvalue>, rvalue: &Rvalue, local: LocalId) -> bool {
    let Rvalue::Aggregate { operands, .. } = rvalue else { return false };
    operands.iter().any(|operand| match operand {
        Operand::Copy(read) | Operand::Move(read) => {
            read.local == local || (read.projection.is_empty() && folded.get(&read.local).is_some_and(|inner| reads_local(folded, inner, local)))
        }
        _ => false,
    })
}

/// The temporaries written straight into the next literal's field: set once, by a literal, in the
/// statement just before one that reads it once, by its own literal, both written one field at a
/// time. A literal that is not itself folded keeps the operands that read the place it writes.
pub(crate) fn foldable(types: &TypeTable, body: &Body) -> HashMap<LocalId, Rvalue> {
    let mut mentions: HashMap<LocalId, usize> = HashMap::new();
    for block in &body.blocks {
        let mut named = Vec::new();
        for stmt in block.stmts.iter().filter(|stmt| !quiet(stmt)) {
            stmt_locals(stmt, &mut named);
        }
        terminator_locals(&block.terminator, &mut named);
        for local in named {
            *mentions.entry(local).or_default() += 1;
        }
    }
    let mut folded: BTreeMap<LocalId, Rvalue> = BTreeMap::new();
    let mut parents: Vec<(LocalId, Rvalue, Vec<LocalId>)> = Vec::new();
    for block in &body.blocks {
        let stmts: Vec<&Stmt> = block.stmts.iter().filter(|stmt| !quiet(stmt)).collect();
        for (at, stmt) in stmts.iter().enumerate() {
            let StmtKind::Assign { place, rvalue: rvalue @ Rvalue::Aggregate { operands, .. } } = &stmt.kind else { continue };
            if !place.projection.is_empty() || !fields_writable(types, body.local(place.local).ty, rvalue) {
                continue;
            }
            let mut children = Vec::new();
            let mut before = at;
            while before > 0 {
                let StmtKind::Assign { place: built, rvalue: inner @ Rvalue::Aggregate { .. } } = &stmts[before - 1].kind else { break };
                let read_here = operands
                    .iter()
                    .any(|operand| matches!(operand, Operand::Copy(read) | Operand::Move(read) if read.local == built.local && read.projection.is_empty()));
                if !built.projection.is_empty()
                    || !read_here
                    || body.local(built.local).kind != LocalKind::Temp
                    || mentions.get(&built.local) != Some(&2)
                    || !fields_writable(types, body.local(built.local).ty, inner)
                {
                    break;
                }
                folded.insert(built.local, inner.clone());
                children.push(built.local);
                before -= 1;
            }
            if !children.is_empty() {
                parents.push((place.local, rvalue.clone(), children));
            }
        }
    }
    for (local, rvalue, children) in parents {
        if !folded.contains_key(&local) && reads_local(&folded, &rvalue, local) {
            for child in children {
                folded.remove(&child);
            }
        }
    }
    folded.into_iter().collect()
}
