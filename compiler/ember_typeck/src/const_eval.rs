//! `[CT-1]`, `[CT-4]` — a `const`'s value, worked out while compiling (V.7).
//!
//! The checked initialiser is reduced to a `Folded` value, each operation done
//! as the target does it: an `f32` sum is rounded to `f32`, an `f16` one to
//! `f16` once (from `double`, as the runtime does), float `//` and `%` are the
//! runtime's floor forms, and an integer is exact in its own width. An
//! overflow, a division by zero or a shift amount out of range is a
//! `Failure::Panic` here, which the checker reports as `E6004` (`[CT-7]`),
//! rather than a panic at every use. Each use is the value's literals
//! (`Folded::to_expr`); a negative integer is `-magnitude`, the form `i64.MIN`
//! has. What this phase cannot evaluate (a call, `**`) is
//! `Failure::Unsupported`.

use ember_hir::{BinOp, Expr, ExprKind, UnOp};
use ember_span::Span;
use ember_types::{EnumId, FloatTy, StructId, Ty, TyKind, TypeTable};

/// A constant's value.
#[derive(Clone)]
pub(crate) struct Folded {
    ty: Ty,
    kind: FoldedKind,
}

#[derive(Clone)]
enum FoldedKind {
    Scalar(Num),
    Str(String),
    Tuple(Vec<Folded>),
    Array(Vec<Folded>),
    Repeat(Box<Folded>, u64),
    Struct(StructId, Vec<Folded>),
    Enum(EnumId, usize, Vec<Folded>),
}

impl Folded {
    /// The value as literals, for one use at `span`.
    pub(crate) fn to_expr(&self, span: Span) -> Expr {
        let ty = self.ty;
        let all = |items: &[Folded]| items.iter().map(|item| item.to_expr(span)).collect();
        let kind = match &self.kind {
            FoldedKind::Scalar(Num::Signed(v)) if *v < 0 => ExprKind::Unary {
                op: UnOp::Neg,
                operand: Box::new(Expr { ty, kind: ExprKind::Int(v.unsigned_abs()), span }),
            },
            FoldedKind::Scalar(Num::Signed(v)) => ExprKind::Int(*v as u128),
            FoldedKind::Scalar(Num::Unsigned(v)) => ExprKind::Int(*v),
            FoldedKind::Scalar(Num::Float(v)) => ExprKind::Float(*v),
            FoldedKind::Scalar(Num::Bool(b)) => ExprKind::Bool(*b),
            FoldedKind::Str(s) => ExprKind::Str(s.clone()),
            FoldedKind::Tuple(items) => ExprKind::TupleLit(all(items)),
            FoldedKind::Array(items) => ExprKind::ArrayLit(all(items)),
            FoldedKind::Repeat(value, count) => ExprKind::ArrayRepeat { value: Box::new(value.to_expr(span)), count: *count },
            FoldedKind::Struct(struct_id, fields) => ExprKind::StructLit { struct_id: *struct_id, fields: all(fields) },
            FoldedKind::Enum(enum_id, variant, fields) => {
                ExprKind::EnumLit { enum_id: *enum_id, variant: *variant, fields: all(fields) }
            }
        };
        Expr { ty, kind, span }
    }
}

/// Why a constant has no value.
pub(crate) enum Failure {
    /// A panic the program would have had, with its message.
    Panic(Span, String),
    /// An operation this phase cannot evaluate while compiling.
    Unsupported(Span),
}

/// A scalar while it is worked on.
#[derive(Clone, Copy)]
enum Num {
    Signed(i128),
    Unsigned(u128),
    Float(f64),
    Bool(bool),
}

/// `expr`, a checked constant initialiser, as a value.
pub(crate) fn fold(types: &TypeTable, expr: &Expr) -> Result<Folded, Failure> {
    let span = expr.span;
    let ty = expr.ty;
    let at = |kind| Folded { ty, kind };
    Ok(match &expr.kind {
        ExprKind::Int(_) | ExprKind::Float(_) | ExprKind::Bool(_) => {
            let value = leaf(types, expr).ok_or(Failure::Unsupported(span))?;
            fitting(types, value, ty, span, "-")?
        }
        ExprKind::Str(s) => at(FoldedKind::Str(s.clone())),
        ExprKind::TupleLit(items) => at(FoldedKind::Tuple(fold_all(types, items)?)),
        ExprKind::ArrayLit(items) => at(FoldedKind::Array(fold_all(types, items)?)),
        ExprKind::ArrayRepeat { value, count } => at(FoldedKind::Repeat(Box::new(fold(types, value)?), *count)),
        ExprKind::StructLit { struct_id, fields } => at(FoldedKind::Struct(*struct_id, fold_all(types, fields)?)),
        ExprKind::EnumLit { enum_id, variant, fields } => at(FoldedKind::Enum(*enum_id, *variant, fold_all(types, fields)?)),
        ExprKind::Field { base, index } => match fold(types, base)?.kind {
            FoldedKind::Struct(_, fields) | FoldedKind::Tuple(fields) => {
                fields.into_iter().nth(*index).ok_or(Failure::Unsupported(span))?
            }
            _ => return Err(Failure::Unsupported(span)),
        },
        // A lossless widening, or a range type read as its representation:
        // the same value in `ty`.
        ExprKind::Widen { expr: inner, .. } | ExprKind::EraseRange(inner) => {
            let value = scalar(&fold(types, inner)?).ok_or(Failure::Unsupported(span))?;
            let value = match (value, types.kind(ty)) {
                (Num::Signed(v), TyKind::Float(_)) => Num::Float(v as f64),
                (Num::Unsigned(v), TyKind::Float(_)) => Num::Float(v as f64),
                (Num::Signed(v), TyKind::Uint(_)) => Num::Unsigned(u128::try_from(v).map_err(|_| Failure::Unsupported(span))?),
                (Num::Unsigned(v), TyKind::Int(_)) => Num::Signed(i128::try_from(v).map_err(|_| Failure::Unsupported(span))?),
                (other, _) => other,
            };
            fitting(types, value, ty, span, "-")?
        }
        ExprKind::Unary { op, operand } => {
            // `-magnitude` is one literal (`-128` at `i8`, whose magnitude
            // alone does not fit).
            if let Some(value) = leaf(types, expr) {
                return fitting(types, value, ty, span, "-");
            }
            let value = scalar(&fold(types, operand)?).ok_or(Failure::Unsupported(span))?;
            unary(types, *op, value, ty, span)?
        }
        ExprKind::Binary { op, lhs, rhs } => {
            let (left, right) = (fold(types, lhs)?, fold(types, rhs)?);
            let a = scalar(&left).ok_or(Failure::Unsupported(span))?;
            let b = scalar(&right).ok_or(Failure::Unsupported(span))?;
            binary(types, *op, a, b, left.ty, ty, span)?
        }
        _ => return Err(Failure::Unsupported(span)),
    })
}

fn fold_all(types: &TypeTable, items: &[Expr]) -> Result<Vec<Folded>, Failure> {
    items.iter().map(|item| fold(types, item)).collect()
}

/// A literal's value: `Int`, `-magnitude` of a signed type, `Float`, `Bool`.
fn leaf(types: &TypeTable, expr: &Expr) -> Option<Num> {
    match (&expr.kind, types.kind(expr.ty)) {
        // A signed value may also be written as its 128-bit two's complement.
        (ExprKind::Int(v), TyKind::Int(_)) => Some(Num::Signed(*v as i128)),
        (ExprKind::Int(v), TyKind::Uint(_)) => Some(Num::Unsigned(*v)),
        (ExprKind::Unary { op: UnOp::Neg, operand }, TyKind::Int(_)) => match operand.kind {
            ExprKind::Int(magnitude) => Some(Num::Signed((magnitude as i128).wrapping_neg())),
            _ => None,
        },
        (ExprKind::Float(v), TyKind::Float(_)) => Some(Num::Float(*v)),
        (ExprKind::Bool(b), _) => Some(Num::Bool(*b)),
        _ => None,
    }
}

/// A folded scalar's value.
fn scalar(value: &Folded) -> Option<Num> {
    match value.kind {
        FoldedKind::Scalar(num) => Some(num),
        _ => None,
    }
}

/// `value` as a scalar of `ty`, a float rounded to it.
fn literal(types: &TypeTable, value: Num, ty: Ty) -> Folded {
    let value = match value {
        Num::Float(v) => Num::Float(round(types, ty, v)),
        other => other,
    };
    Folded { ty, kind: FoldedKind::Scalar(value) }
}

/// `value` as a scalar of `ty`, or the overflow of `op` if it does not fit.
fn fitting(types: &TypeTable, value: Num, ty: Ty, span: Span, op: &str) -> Result<Folded, Failure> {
    let fits = match value {
        Num::Signed(v) => {
            let least = ember_types::signed_min_magnitude(types, ty).map(|m| (m as i128).wrapping_neg());
            let most = ember_types::int_max(types, ty).map(|m| m as i128);
            matches!((least, most), (Some(least), Some(most)) if least <= v && v <= most)
        }
        Num::Unsigned(v) => ember_types::int_max(types, ty).is_some_and(|most| v <= most),
        Num::Float(_) | Num::Bool(_) => true,
    };
    if !fits {
        return Err(Failure::Panic(span, format!("integer overflow in `{op}`")));
    }
    Ok(literal(types, value, ty))
}

/// A float rounded to `ty`, as the target keeps it.
fn round(types: &TypeTable, ty: Ty, v: f64) -> f64 {
    match types.kind(ty) {
        TyKind::Float(FloatTy::F32) => f64::from(v as f32),
        TyKind::Float(FloatTy::F16) => ember_types::f16_value(ember_types::f16_bits(v)),
        _ => v,
    }
}

fn unary(types: &TypeTable, op: UnOp, value: Num, ty: Ty, span: Span) -> Result<Folded, Failure> {
    let overflow = || Failure::Panic(span, "integer overflow in `-`".to_string());
    let result = match (op, value) {
        (UnOp::Neg, Num::Signed(v)) => Num::Signed(v.checked_neg().ok_or_else(overflow)?),
        // D-314 — only 0 has an unsigned negation.
        (UnOp::Neg, Num::Unsigned(0)) => Num::Unsigned(0),
        (UnOp::Neg, Num::Unsigned(_)) => return Err(overflow()),
        (UnOp::Neg, Num::Float(v)) => Num::Float(-v),
        (UnOp::Not, Num::Bool(b)) => Num::Bool(!b),
        (UnOp::BitNot, Num::Signed(v)) => Num::Signed(!v),
        (UnOp::BitNot, Num::Unsigned(v)) => Num::Unsigned(!v & mask(types, ty)),
        _ => return Err(Failure::Unsupported(span)),
    };
    fitting(types, result, ty, span, "-")
}

/// Every bit of an unsigned type.
fn mask(types: &TypeTable, ty: Ty) -> u128 {
    ember_types::int_max(types, ty).unwrap_or(u128::MAX)
}

fn binary(types: &TypeTable, op: BinOp, a: Num, b: Num, operand_ty: Ty, ty: Ty, span: Span) -> Result<Folded, Failure> {
    let spelling = op.spelling();
    let overflow = || Failure::Panic(span, format!("integer overflow in `{spelling}`"));
    let by_zero = || Failure::Panic(span, "division by zero".to_string());
    let compare = |ordering: Option<std::cmp::Ordering>| -> Result<Folded, Failure> {
        use std::cmp::Ordering::{Equal, Greater, Less};
        let holds = match (op, ordering) {
            (BinOp::Eq, found) => found == Some(Equal),
            (BinOp::Ne, found) => found != Some(Equal),
            (BinOp::Lt, found) => found == Some(Less),
            (BinOp::Le, found) => matches!(found, Some(Less | Equal)),
            (BinOp::Gt, found) => found == Some(Greater),
            (BinOp::Ge, found) => matches!(found, Some(Greater | Equal)),
            _ => return Err(Failure::Unsupported(span)),
        };
        Ok(literal(types, Num::Bool(holds), ty))
    };
    if op.is_comparison() {
        return match (a, b) {
            (Num::Signed(x), Num::Signed(y)) => compare(Some(x.cmp(&y))),
            (Num::Unsigned(x), Num::Unsigned(y)) => compare(Some(x.cmp(&y))),
            // IEEE: a NaN is unordered, so only `!=` holds of it.
            (Num::Float(x), Num::Float(y)) => compare(x.partial_cmp(&y)),
            (Num::Bool(x), Num::Bool(y)) => compare(Some(x.cmp(&y))),
            _ => Err(Failure::Unsupported(span)),
        };
    }
    let result = match (a, b) {
        (Num::Bool(x), Num::Bool(y)) => Num::Bool(match op {
            BinOp::And => x && y,
            BinOp::Or => x || y,
            BinOp::BitAnd => x & y,
            BinOp::BitOr => x | y,
            BinOp::BitXor => x ^ y,
            _ => return Err(Failure::Unsupported(span)),
        }),
        // `[TYP-10]` — the amount may be any integer type; one outside
        // `0 ≤ n < width` panics, as the runtime's check does.
        (value, amount) if matches!(op, BinOp::Shl | BinOp::Shr) => {
            let width = ember_types::bit_width(types, operand_ty).ok_or(Failure::Unsupported(span))?;
            let n = match amount {
                Num::Signed(n) if n >= 0 => n as u128,
                Num::Unsigned(n) => n,
                Num::Signed(_) => width.into(),
                _ => return Err(Failure::Unsupported(span)),
            };
            if n >= u128::from(width) {
                return Err(Failure::Panic(span, "integer overflow in `shift`".to_string()));
            }
            let n = n as u32;
            match (value, op) {
                // Bits shifted out are dropped, and the result is read back
                // at the type's width.
                (Num::Signed(v), BinOp::Shl) => {
                    let shift = 128 - width as u32;
                    Num::Signed((((v as u128) << n) << shift) as i128 >> shift)
                }
                (Num::Signed(v), _) => Num::Signed(v >> n),
                (Num::Unsigned(v), BinOp::Shl) => Num::Unsigned((v << n) & mask(types, ty)),
                (Num::Unsigned(v), _) => Num::Unsigned(v >> n),
                _ => return Err(Failure::Unsupported(span)),
            }
        }
        (Num::Signed(x), Num::Signed(y)) => Num::Signed(match op {
            BinOp::Add => x.checked_add(y).ok_or_else(overflow)?,
            BinOp::Sub => x.checked_sub(y).ok_or_else(overflow)?,
            BinOp::Mul => x.checked_mul(y).ok_or_else(overflow)?,
            BinOp::FloorDiv => {
                if y == 0 {
                    return Err(by_zero());
                }
                let q = x.checked_div(y).ok_or_else(overflow)?;
                if x % y != 0 && ((x < 0) != (y < 0)) { q - 1 } else { q }
            }
            BinOp::FloorRem => {
                if y == 0 {
                    return Err(by_zero());
                }
                // `MIN % -1` is 0, which Rust's `%` cannot say.
                let r = if y == -1 { 0 } else { x % y };
                if r != 0 && ((r < 0) != (y < 0)) { r + y } else { r }
            }
            BinOp::BitAnd => x & y,
            BinOp::BitOr => x | y,
            BinOp::BitXor => x ^ y,
            _ => return Err(Failure::Unsupported(span)),
        }),
        (Num::Unsigned(x), Num::Unsigned(y)) => Num::Unsigned(match op {
            BinOp::Add => x.checked_add(y).ok_or_else(overflow)?,
            BinOp::Sub => x.checked_sub(y).ok_or_else(overflow)?,
            BinOp::Mul => x.checked_mul(y).ok_or_else(overflow)?,
            BinOp::FloorDiv | BinOp::FloorRem if y == 0 => return Err(by_zero()),
            BinOp::FloorDiv => x / y,
            BinOp::FloorRem => x % y,
            BinOp::BitAnd => x & y,
            BinOp::BitOr => x | y,
            BinOp::BitXor => x ^ y,
            _ => return Err(Failure::Unsupported(span)),
        }),
        (Num::Float(x), Num::Float(y)) => Num::Float(match op {
            BinOp::Add => x + y,
            BinOp::Sub => x - y,
            BinOp::Mul => x * y,
            BinOp::Div => x / y,
            BinOp::FloorDiv | BinOp::FloorRem => {
                let single = matches!(types.kind(ty), TyKind::Float(FloatTy::F32));
                let (div, rem) = if single {
                    let (d, r) = floor_div_rem_f32(x as f32, y as f32);
                    (f64::from(d), f64::from(r))
                } else {
                    floor_div_rem_f64(x, y)
                };
                if op == BinOp::FloorDiv { div } else { rem }
            }
            _ => return Err(Failure::Unsupported(span)),
        }),
        _ => return Err(Failure::Unsupported(span)),
    };
    fitting(types, result, ty, span, spelling)
}

/// The runtime's `ember_floordiv_f64` and `ember_floorrem_f64`: Python's float
/// `//` and `%` (ODR-021). An `f16`'s go through these, as its every
/// operation goes through `double`.
fn floor_div_rem_f64(a: f64, b: f64) -> (f64, f64) {
    let modulo = a % b;
    let rem = if modulo != 0.0 {
        if (b < 0.0) != (modulo < 0.0) { modulo + b } else { modulo }
    } else {
        0.0f64.copysign(b)
    };
    let mut div = (a - modulo) / b;
    if modulo != 0.0 && ((b < 0.0) != (modulo < 0.0)) {
        div -= 1.0;
    }
    let div = if div != 0.0 {
        let floor = div.floor();
        if div - floor > 0.5 { floor + 1.0 } else { floor }
    } else {
        0.0f64.copysign(a / b)
    };
    (div, rem)
}

/// The same in `f32`, as `ember_floordiv_f32` and `ember_floorrem_f32` are.
fn floor_div_rem_f32(a: f32, b: f32) -> (f32, f32) {
    let modulo = a % b;
    let rem = if modulo != 0.0 {
        if (b < 0.0) != (modulo < 0.0) { modulo + b } else { modulo }
    } else {
        0.0f32.copysign(b)
    };
    let mut div = (a - modulo) / b;
    if modulo != 0.0 && ((b < 0.0) != (modulo < 0.0)) {
        div -= 1.0;
    }
    let div = if div != 0.0 {
        let floor = div.floor();
        if div - floor > 0.5 { floor + 1.0 } else { floor }
    } else {
        0.0f32.copysign(a / b)
    };
    (div, rem)
}
