//! `[FFI-21a]` — a function C may call does not return a pointer into a
//! parameter C passes as a copy (`E5042`).
//!
//! Called from Ember, a borrowed parameter is the caller's own value (`[FN-1]`);
//! the C entry a function gets when its value becomes an `extern "C" fn`
//! (`[FN-6]`) receives a by-value parameter as C passes it, a copy of its own,
//! and lends the function that copy, which ends when the call returns. A raw
//! pointer the function returns that it made from a reference to such a
//! parameter would point into that copy. Followed: copies, conversions,
//! references to the parameter's parts and aggregates holding such a pointer;
//! not calls to other functions, and not values read through the pointer (they
//! are data, not addresses).

use std::collections::{BTreeMap, BTreeSet};

use ember_diag::{Diagnostic, Sink, codes};
use ember_mir::{Body, CastKind, Const, LocalId, Operand, Place, Projection, Rvalue, StmtKind, Terminator};
use ember_span::Span;
use ember_types::{FnParamMode, Ty, TyKind, TypeTable};

use crate::regions::place_type;

/// The callable types the program turns into `extern "C" fn`, each with the
/// first place it does.
pub fn callable_types_given_to_c(bodies: &[Body], types: &TypeTable) -> BTreeMap<Ty, Span> {
    let mut given = BTreeMap::new();
    for body in bodies {
        for stmt in body.blocks.iter().flat_map(|block| &block.stmts) {
            if let StmtKind::Assign { rvalue: Rvalue::Cast { kind: CastKind::FnToC, operand: Operand::Copy(from) | Operand::Move(from), .. }, .. } =
                &stmt.kind
            {
                given.entry(place_type(body, types, from)).or_insert(stmt.span);
            }
        }
    }
    given
}

/// `E5042` for each function whose value has a callable type the program
/// gives C and that returns a raw pointer into a borrowed by-value parameter.
pub fn check_c_entry_returns_all(bodies: &[Body], types: &TypeTable, sink: &mut Sink) -> usize {
    let given = callable_types_given_to_c(bodies, types);
    if given.is_empty() {
        return 0;
    }
    // The functions whose values have one of those types: C may call each.
    let mut reachable: BTreeMap<&str, (Ty, Span)> = BTreeMap::new();
    for body in bodies {
        for stmt in body.blocks.iter().flat_map(|block| &block.stmts) {
            if let StmtKind::Assign { place, rvalue: Rvalue::Use(Operand::Const(Const::Fn(symbol))) } = &stmt.kind {
                let ty = place_type(body, types, place);
                if let Some(&span) = given.get(&ty) {
                    reachable.entry(symbol.as_str()).or_insert((ty, span));
                }
            }
        }
    }
    let mut reported = 0;
    for body in bodies {
        let Some(&(ty, coerced_at)) = reachable.get(body.symbol.as_str()) else { continue };
        let TyKind::Fn { params, .. } = types.kind(ty) else { continue };
        // A borrowed parameter C passes by value: the C entry lends the
        // function its own copy (`passed_by_address`).
        let copied: BTreeSet<LocalId> = params
            .iter()
            .enumerate()
            .filter(|(_, param)| param.mode == FnParamMode::Borrow && types.passed_by_address(param.ty))
            .map(|(index, _)| LocalId(index as u32 + 1))
            .collect();
        if copied.is_empty() {
            continue;
        }
        if let Some((parameter, span)) = returned_copy_pointer(body, types, &copied) {
            let name = body.locals[parameter.0 as usize].name.clone().unwrap_or_else(|| format!("parameter {}", parameter.0));
            sink.emit(
                Diagnostic::error(
                    codes::E5042,
                    span,
                    format!("`{}` returns a pointer into `{name}`, which C passes as a copy", body.name),
                )
                .primary_label("the pointer is returned here")
                .secondary(coerced_at, "its value becomes an `extern \"C\" fn` here, so C may call it")
                .note(format!(
                    "called through C, `{name}` is the call's own copy, which ends when the call returns: the pointer would \
                     point into it [FFI-21a]"
                ))
                .help(format!("take `{name}` as a pointer (`*T`), so C gets back the pointer it passed")),
            );
            reported += 1;
        }
    }
    reported
}

/// The first parameter of `copied` whose address can reach the return value,
/// with the statement that puts it there: a forward may-analysis of which
/// locals hold an address into one of those parameters.
fn returned_copy_pointer(body: &Body, types: &TypeTable, copied: &BTreeSet<LocalId>) -> Option<(LocalId, Span)> {
    if body.blocks.is_empty() {
        return None;
    }
    // Per local holding such an address: the parameter, and where the
    // return local got it.
    type State = BTreeMap<LocalId, (LocalId, Span)>;
    let entry: State = copied.iter().map(|&param| (param, (param, Span::DUMMY))).collect();
    let mut entries: Vec<Option<State>> = vec![None; body.blocks.len()];
    entries[0] = Some(entry);
    let mut work = vec![0usize];
    while let Some(index) = work.pop() {
        let Some(mut state) = entries[index].clone() else { continue };
        let block = &body.blocks[index];
        for stmt in &block.stmts {
            let StmtKind::Assign { place, rvalue } = &stmt.kind else { continue };
            let origin = rvalue_origin(body, types, rvalue, &state).map(|(param, _)| (param, stmt.span));
            match (origin, place.projection.is_empty()) {
                (Some(origin), _) => {
                    state.insert(place.local, origin);
                }
                // A whole new value: no longer such an address.
                (None, true) => {
                    state.remove(&place.local);
                }
                (None, false) => {}
            }
        }
        match &block.terminator {
            Terminator::Return if holds_address(types, body.local(LocalId(0)).ty, 0) => {
                if let Some(&(param, span)) = state.get(&LocalId(0)) {
                    return Some((param, span));
                }
            }
            Terminator::Call { dest, .. } if dest.projection.is_empty() => {
                state.remove(&dest.local);
            }
            _ => {}
        }
        for next in successors(&block.terminator) {
            let merged = match &entries[next] {
                Some(existing) => {
                    let mut merged = existing.clone();
                    for (local, origin) in &state {
                        merged.entry(*local).or_insert(*origin);
                    }
                    merged
                }
                None => state.clone(),
            };
            if entries[next].as_ref() != Some(&merged) {
                entries[next] = Some(merged);
                work.push(next);
            }
        }
    }
    None
}

/// Whether an rvalue is an address into a copied parameter: the parameter's
/// own value, a reference to a part of it, a copy or conversion of such an
/// address, or an aggregate holding one. A value read through it is not.
fn rvalue_origin(
    body: &Body,
    types: &TypeTable,
    rvalue: &Rvalue,
    state: &BTreeMap<LocalId, (LocalId, Span)>,
) -> Option<(LocalId, Span)> {
    let address = |place: &Place| {
        // Read through a pointer: data, not an address. A part of an
        // aggregate holding such an address is one only if it can hold one.
        if place.projection.iter().any(|step| matches!(step, Projection::Deref))
            || !holds_address(types, place_type(body, types, place), 0)
        {
            return None;
        }
        state.get(&place.local).copied()
    };
    let operand = |operand: &Operand| match operand {
        Operand::Copy(place) | Operand::Move(place) => address(place),
        Operand::Const(_) => None,
    };
    match rvalue {
        Rvalue::Use(o) | Rvalue::Cast { operand: o, .. } => operand(o),
        Rvalue::Aggregate { operands, .. } => operands.iter().find_map(operand),
        // `&(*p).field`: a part of what `p` points at. Past a second
        // dereference it is where a pointer stored in the copy points.
        Rvalue::Ref { place, .. }
            if matches!(place.projection.first(), Some(Projection::Deref))
                && !place.projection[1..].iter().any(|step| matches!(step, Projection::Deref)) =>
        {
            state.get(&place.local).copied()
        }
        Rvalue::Ref { .. } => None,
        Rvalue::BinaryOp { .. } | Rvalue::UnaryOp { .. } | Rvalue::Repeat { .. } | Rvalue::Discriminant(_) => None,
    }
}

/// Whether a value of this type can hold an address: a pointer, a reference,
/// or an aggregate with one.
fn holds_address(types: &TypeTable, ty: Ty, depth: usize) -> bool {
    if depth > 16 {
        return true;
    }
    match types.kind(ty) {
        TyKind::Ptr { .. } | TyKind::Ref { .. } => true,
        TyKind::Struct(id) => types.struct_def(*id).fields.iter().any(|field| holds_address(types, field.ty, depth + 1)),
        TyKind::Tuple(items) => items.iter().any(|&item| holds_address(types, item, depth + 1)),
        TyKind::Array { elem, .. } => holds_address(types, *elem, depth + 1),
        TyKind::Enum(id) => types.enum_def(*id).variants.iter().any(|variant| {
            variant.fields.iter().any(|field| holds_address(types, field.ty, depth + 1))
        }),
        _ => false,
    }
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
