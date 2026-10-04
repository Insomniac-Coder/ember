//! `[RC-6]` — every retain and release that survives inside a loop, with the
//! reason it could not be removed: a surviving count operation blocks
//! vectorisation.
//!
//! It reads the final MIR, after every elision, and mirrors what the backend
//! emits. A counted value is one that is `Copy` and still needs a drop: a
//! class or interface handle, a `Shared` or `Weak`, or a value holding one.
//! Copying one retains (an assignment's copy, a literal's copied field or
//! element, an upcast's copy, a copy passed to an `owned` parameter), unless
//! it lands in an uncounted handle (`[RC-3]`); dropping one releases.

use std::collections::HashMap;

use ember_mir::{Body, Builtin, CastKind, FuncRef, Operand, ParameterMode, Place, Rvalue, StmtKind, Terminator};
use ember_span::Span;
use ember_types::{FnParamMode, Ty, TyKind, TypeTable};

use crate::range_facts::{loop_headers, natural_loops, reverse_postorder};
use crate::regions::place_type;

/// One count operation inside a loop.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CountOperation {
    /// The function's source name.
    pub function: String,
    /// Where the operation is.
    pub span: Span,
    /// The innermost loop it is in: the span of that loop's test.
    pub loop_span: Span,
    /// A retain, or else a release.
    pub retain: bool,
    pub reason: String,
}

/// Every count operation inside a loop in `bodies`, in body and block order.
pub fn surviving_count_operations(bodies: &[Body], types: &TypeTable) -> Vec<CountOperation> {
    let modes: HashMap<&str, &[ParameterMode]> =
        bodies.iter().map(|body| (body.symbol.as_str(), body.param_modes.as_slice())).collect();
    let mut found = Vec::new();
    for body in bodies.iter().filter(|body| !body.blocks.is_empty()) {
        let order = reverse_postorder(body);
        let headers = loop_headers(body, &order);
        if headers.is_empty() {
            continue;
        }
        // Each block's innermost loop: the smallest one holding it.
        let loops = natural_loops(body, &order, &headers);
        let mut innermost: HashMap<usize, (usize, usize)> = HashMap::new();
        for (&header, blocks) in &loops {
            for &block in blocks {
                let size = blocks.len();
                if innermost.get(&block).is_none_or(|&(_, best)| size < best) {
                    innermost.insert(block, (header, size));
                }
            }
        }
        let mut blocks: Vec<usize> = innermost.keys().copied().collect();
        blocks.sort_unstable();
        for block in blocks {
            let loop_span = body.blocks[innermost[&block].0].terminator_span;
            let data = &body.blocks[block];
            let mut note = |span: Span, retain: bool, reason: String| {
                found.push(CountOperation { function: body.name.clone(), span, loop_span, retain, reason });
            };
            for (at, stmt) in data.stmts.iter().enumerate() {
                match &stmt.kind {
                    StmtKind::Assign { place, rvalue } => {
                        for reason in retains(body, types, place, rvalue, &data.stmts[at + 1..]) {
                            note(stmt.span, true, reason);
                        }
                    }
                    StmtKind::Drop { place, .. } if counted(types, place_type(body, types, place)) && !uncounted(body, place) => {
                        let reason = if place.projection.is_empty() {
                            format!("{} ends here, releasing its count", described(body, place))
                        } else {
                            format!("the value in {} is released as a store replaces it ([OWN-5])", described(body, place))
                        };
                        note(stmt.span, false, reason);
                    }
                    _ => {}
                }
            }
            // `Array.push` copies a `Copy` value into the list (the backend's
            // `[RC-1]` retain before `ember_vec_push`).
            if let Terminator::Call { func: FuncRef::Builtin { which: Builtin::ArrayPush, .. }, args, .. } = &data.terminator
                && let Some(Operand::Copy(place)) = args.get(1)
                && counted(types, place_type(body, types, place))
            {
                note(
                    data.terminator_span,
                    true,
                    format!("{} is pushed into a list, which keeps a count of its own", described(body, place)),
                );
            }
            if let Terminator::Call { func, args, .. } = &data.terminator {
                for (index, argument) in args.iter().enumerate() {
                    let Operand::Copy(place) = argument else { continue };
                    if counted(types, place_type(body, types, place)) && owned_argument(types, body, &modes, func, index) {
                        note(
                            data.terminator_span,
                            true,
                            format!("{} is passed to an `owned` parameter, which ends its own count", described(body, place)),
                        );
                    }
                }
            }
        }
    }
    found
}

/// A value whose copy is a count operation: `Copy`, yet with a drop.
fn counted(types: &TypeTable, ty: Ty) -> bool {
    types.is_copy(ty) && types.needs_drop(ty)
}

/// `[RC-3]` — a handle the uncounted-handle analysis marked: its copies and
/// its end are no count operations.
fn uncounted(body: &Body, place: &Place) -> bool {
    place.projection.is_empty() && body.uncounted_handles.contains(&place.local)
}

/// The retains an assignment makes, each with its reason. `after` is the rest
/// of its block: a copy into a temporary the block then moves into a field or
/// element is that store (the `[OWN-5]` assignment's own temporary).
fn retains(body: &Body, types: &TypeTable, place: &Place, rvalue: &Rvalue, after: &[ember_mir::Stmt]) -> Vec<String> {
    let stored_into = || {
        if !place.projection.is_empty() || body.local(place.local).name.is_some() {
            return None;
        }
        after.iter().find_map(|stmt| match &stmt.kind {
            StmtKind::Assign { place: target, rvalue: Rvalue::Use(Operand::Move(moved)) }
                if moved == place && !target.projection.is_empty() =>
            {
                Some(target.clone())
            }
            _ => None,
        })
    };
    match rvalue {
        Rvalue::Use(Operand::Copy(source)) if counted(types, place_type(body, types, place)) && !uncounted(body, place) => {
            vec![if let Some(target) = stored_into() {
                format!(
                    "{} is stored into {}, which keeps a count of its own",
                    described(body, source),
                    described(body, &target)
                )
            } else if place.projection.is_empty() {
                format!(
                    "{} is copied into {}, which keeps a count of its own: nothing shows that its source holds the object while the copy lives ([RC-3])",
                    described(body, source),
                    described(body, place)
                )
            } else {
                format!(
                    "{} is stored into {}, a field or element, which keeps a count of its own",
                    described(body, source),
                    described(body, place)
                )
            }]
        }
        Rvalue::Aggregate { operands, .. } => operands
            .iter()
            .filter_map(|operand| match operand {
                Operand::Copy(source) if counted(types, place_type(body, types, source)) => Some(format!(
                    "{} is copied into a value being built (a field or element of a literal)",
                    described(body, source)
                )),
                _ => None,
            })
            .collect(),
        Rvalue::Cast { kind: CastKind::ClassUpcast | CastKind::ClassInterfaceUpcast { .. }, operand: Operand::Copy(source), .. } => {
            vec![format!("{} is copied by an upcast", described(body, source))]
        }
        _ => Vec::new(),
    }
}

/// Whether a call's `index`th argument goes to an `owned` parameter, as the
/// backend decides it (`call_argument_is_owned`).
fn owned_argument(types: &TypeTable, body: &Body, modes: &HashMap<&str, &[ParameterMode]>, func: &FuncRef, index: usize) -> bool {
    match func {
        FuncRef::Direct { symbol, .. } => {
            modes.get(symbol.as_str()).and_then(|modes| modes.get(index)).is_some_and(|mode| *mode == ParameterMode::Owned)
        }
        FuncRef::Indirect { operand, .. } => {
            let (Operand::Copy(place) | Operand::Move(place)) = operand else { return false };
            let TyKind::Fn { params, .. } = types.kind(place_type(body, types, place)) else { return false };
            params.get(index).is_some_and(|parameter| parameter.mode == FnParamMode::Owned)
        }
        FuncRef::Interface { param_modes, .. } => index
            .checked_sub(1)
            .and_then(|parameter| param_modes.get(parameter))
            .is_some_and(|mode| *mode == ParameterMode::Owned),
        FuncRef::Virtual { param_modes, .. } => param_modes.get(index).is_some_and(|mode| *mode == ParameterMode::Owned),
        _ => false,
    }
}

/// A place in words: `` `x` ``, `` a field or element of `x` ``, or a
/// compiler temporary.
fn described(body: &Body, place: &Place) -> String {
    let name = body.local(place.local).name.clone();
    match (name, place.projection.is_empty()) {
        (Some(name), true) => format!("`{name}`"),
        (Some(name), false) => format!("a field or element of `{name}`"),
        (None, _) => "a temporary".to_string(),
    }
}
