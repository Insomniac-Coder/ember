//! Integer reductions over a read-only, unit-stride byte stream. The MIR
//! has already proved or versioned away its bounds and arithmetic checks.
//! This selection adds no proof and never handles a checked operation.

use super::ForLoop;
use ember_mir::{BinOp, Body, CastKind, LocalId, Operand, Place, Projection, Rvalue, StmtKind, Terminator};
use ember_types::{IntTy, Ty, TyKind, TypeTable, UintTy};
use ember_span::Span;
use std::collections::{BTreeMap, BTreeSet};

pub(super) struct ByteSum {
    pub total: LocalId,
    pub element: Place,
    pub signed: bool,
    pub subtract: bool,
    pub span: Span,
}

#[derive(Clone)]
enum Value {
    Counter,
    Reference(Place, bool),
    Byte(Place, bool),
    Total(LocalId),
}

fn wide(types: &TypeTable, ty: Ty) -> bool {
    matches!(types.kind(ty), TyKind::Int(IntTy::I64) | TyKind::Uint(UintTy::U64))
}

/// Select by the complete data flow, not by an expression's spelling: copies
/// of the index, element references, and widening casts all carry the same
/// stream. Every other write must be a dead, loop-scoped intermediate. Any
/// call, check, mutation, live result, second total, address-taken total/counter,
/// or unsupported computation leaves the original loop intact.
pub(super) fn byte_sum(
    counted: &ForLoop,
    body: &Body,
    types: &TypeTable,
    locals: &[usize],
    address_taken: &BTreeSet<LocalId>,
    place_ty: impl Fn(&Place) -> Ty,
) -> Option<ByteSum> {
    if counted.inclusive
        || address_taken.contains(&counted.counter)
        || !matches!(types.kind(body.local(counted.counter).ty), TyKind::Uint(UintTy::Usize))
        || body.local(counted.limit).ty != body.local(counted.counter).ty
    {
        return None;
    }
    let mut values = BTreeMap::from([(counted.counter, Value::Counter)]);
    let mut result = None;
    for (position, &block) in counted.chain.iter().enumerate() {
        let data = &body.blocks[block];
        if !matches!(data.terminator, Terminator::Goto(_)) {
            return None;
        }
        for (index, stmt) in data.stmts.iter().enumerate() {
            if position + 1 == counted.chain.len() && index == counted.increment {
                continue;
            }
            let (place, rvalue) = match &stmt.kind {
                StmtKind::Assign { place, rvalue } => (place, rvalue),
                StmtKind::StorageLive(_) | StmtKind::StorageDead(_) | StmtKind::Nop => continue,
                _ => return None,
            };
            if !place.projection.is_empty() || place.local == counted.counter || place.local == counted.limit {
                return None;
            }
            if let Rvalue::BinaryOp { op, lhs, rhs } = rvalue {
                if result.is_some() || !wide(types, body.local(place.local).ty) {
                    return None;
                }
                let left = operand(lhs, &values, types, body, &place_ty, counted.counter)?;
                let right = operand(rhs, &values, types, body, &place_ty, counted.counter)?;
                let (total, element, signed, subtract) = match (op, left, right) {
                    (BinOp::Add, Value::Total(total), Value::Byte(element, signed))
                    | (BinOp::Add, Value::Byte(element, signed), Value::Total(total)) => (total, element, signed, false),
                    (BinOp::Sub, Value::Total(total), Value::Byte(element, signed)) => (total, element, signed, true),
                    _ => return None,
                };
                if total != place.local || locals.contains(&(total.0 as usize))
                    || address_taken.contains(&total)
                {
                    return None;
                }
                result = Some(ByteSum { total, element, signed, subtract, span: stmt.span });
                values.insert(total, Value::Total(total));
                continue;
            }
            if !locals.contains(&(place.local.0 as usize)) {
                return None;
            }
            let value = match rvalue {
                Rvalue::Use(value) => operand(value, &values, types, body, &place_ty, counted.counter)?,
                Rvalue::Ref { place, mutable: false } => {
                    let (place, signed) = stream(place, &values, types, &place_ty, counted.counter)?;
                    Value::Reference(place, signed)
                }
                Rvalue::Cast { kind: CastKind::Numeric, operand: value, to } => {
                    let value = operand(value, &values, types, body, &place_ty, counted.counter)?;
                    match value {
                        Value::Counter if body.local(place.local).ty == body.local(counted.counter).ty => value,
                        Value::Byte(_, _) if wide(types, *to) => value,
                        // Narrowing or changing an intermediate's sign can
                        // change its value before the final sum.
                        _ => return None,
                    }
                }
                _ => return None,
            };
            values.insert(place.local, value);
        }
    }
    result
}

fn operand(
    operand: &Operand,
    values: &BTreeMap<LocalId, Value>,
    types: &TypeTable,
    body: &Body,
    place_ty: &impl Fn(&Place) -> Ty,
    counter: LocalId,
) -> Option<Value> {
    let (Operand::Copy(place) | Operand::Move(place)) = operand else { return None };
    if place.projection.is_empty() {
        return values.get(&place.local).cloned().or_else(|| {
            wide(types, body.local(place.local).ty).then_some(Value::Total(place.local))
        });
    }
    if place.projection.as_slice() == [Projection::Deref]
        && let Some(Value::Reference(source, signed)) = values.get(&place.local)
    {
        return Some(Value::Byte(source.clone(), *signed));
    }
    stream(place, values, types, place_ty, counter).map(|(place, signed)| Value::Byte(place, signed))
}

fn stream(
    place: &Place,
    values: &BTreeMap<LocalId, Value>,
    types: &TypeTable,
    place_ty: &impl Fn(&Place) -> Ty,
    counter: LocalId,
) -> Option<(Place, bool)> {
    let Some(Projection::Index(index)) = place.projection.last() else { return None };
    if !matches!(values.get(index), Some(Value::Counter)) {
        return None;
    }
    let signed = match types.kind(place_ty(place)) {
        TyKind::Uint(UintTy::U8) => false,
        TyKind::Int(IntTy::I8) => true,
        _ => return None,
    };
    // No other indexed projection, nor a base computed inside this loop:
    // only the last projection advances over an invariant stream.
    if values.contains_key(&place.local)
        || place.projection[..place.projection.len() - 1].iter().any(|p| matches!(p, Projection::Index(_)))
    {
        return None;
    }
    let mut source = place.clone();
    *source.projection.last_mut()? = Projection::Index(counter);
    Some((source, signed))
}
