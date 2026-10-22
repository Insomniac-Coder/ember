//! Numbers as the target computes them (`[CT-4]`): integers of each width, wrapping and checked;
//! Python's floor division (`[TYP-28]`, `[TYP-29]`); the conversions of `[TYP-6]`; and `f16`
//! carried as its bits, worked in `f64` and rounded once (D-316), as the runtime does.

use ember_mir::BinOp;

use crate::wide::{self, W256};

/// An integer type: its width in bits and whether it is signed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IntKind {
    pub bits: u32,
    pub signed: bool,
}

impl IntKind {
    pub fn bytes(self) -> usize {
        (self.bits / 8) as usize
    }

    pub fn mask(self) -> u128 {
        if self.bits >= 128 { u128::MAX } else { (1u128 << self.bits) - 1 }
    }

    /// The value of `bytes` (little-endian, this width, at most 128 bits) as an `i128`,
    /// sign-extended when signed. An unsigned 128-bit value above `i128::MAX` wraps; use
    /// `unsigned` for those.
    pub fn signed_value(self, bytes: &[u8]) -> i128 {
        let raw = crate::memory::uint(&bytes[..self.bytes().min(16)]);
        if self.signed && self.bits < 128 {
            let shift = 128 - self.bits;
            ((raw << shift) as i128) >> shift
        } else {
            raw as i128
        }
    }

    /// The value of bits `raw` (within the width), sign-extended when signed.
    pub fn extend(self, raw: u128) -> i128 {
        if self.signed && self.bits < 128 {
            let shift = 128 - self.bits;
            ((raw << shift) as i128) >> shift
        } else {
            raw as i128
        }
    }

    pub fn unsigned(self, bytes: &[u8]) -> u128 {
        crate::memory::uint(&bytes[..self.bytes().min(16)]) & self.mask()
    }

    /// The low bits of `value`, at this width; a 256-bit one zero-extended.
    pub fn encode(self, value: u128) -> Vec<u8> {
        let mut bytes = (value & self.mask()).to_le_bytes().to_vec();
        bytes.resize(self.bytes(), 0);
        bytes
    }

    fn min(self) -> i128 {
        if !self.signed { 0 } else if self.bits >= 128 { i128::MIN } else { -(1i128 << (self.bits - 1)) }
    }

    fn max(self) -> u128 {
        if self.signed { self.mask() >> 1 } else { self.mask() }
    }

    /// Whether the exact result `value` (an unbounded integer, as a sign and magnitude in
    /// `i128` where it fits) is representable.
    fn fits(self, value: Wide) -> bool {
        match value {
            Wide::Neg(m) => self.signed && m <= self.min().unsigned_abs(),
            Wide::Pos(m) => m <= self.max(),
            Wide::Overflow => false,
        }
    }
}

/// An exact integer result: a magnitude and its sign, or one past every 128-bit magnitude.
#[derive(Clone, Copy, Debug)]
enum Wide {
    Pos(u128),
    Neg(u128),
    Overflow,
}

impl Wide {
    fn of(kind: IntKind, bytes: &[u8]) -> Wide {
        if kind.signed {
            let v = kind.signed_value(bytes);
            if v < 0 { Wide::Neg(v.unsigned_abs()) } else { Wide::Pos(v as u128) }
        } else {
            Wide::Pos(kind.unsigned(bytes))
        }
    }

    fn neg(self) -> Wide {
        match self {
            Wide::Pos(0) => Wide::Pos(0),
            Wide::Pos(m) => Wide::Neg(m),
            Wide::Neg(m) => Wide::Pos(m),
            Wide::Overflow => Wide::Overflow,
        }
    }

    fn add(self, other: Wide) -> Wide {
        match (self, other) {
            (Wide::Pos(a), Wide::Pos(b)) => a.checked_add(b).map_or(Wide::Overflow, Wide::Pos),
            (Wide::Neg(a), Wide::Neg(b)) => a.checked_add(b).map_or(Wide::Overflow, Wide::Neg),
            (Wide::Pos(a), Wide::Neg(b)) | (Wide::Neg(b), Wide::Pos(a)) => {
                if a >= b { Wide::Pos(a - b) } else { Wide::Neg(b - a) }
            }
            _ => Wide::Overflow,
        }
    }

    fn mul(self, other: Wide) -> Wide {
        let (negative, a, b) = match (self, other) {
            (Wide::Pos(a), Wide::Pos(b)) | (Wide::Neg(a), Wide::Neg(b)) => (false, a, b),
            (Wide::Pos(a), Wide::Neg(b)) | (Wide::Neg(a), Wide::Pos(b)) => (true, a, b),
            _ => return Wide::Overflow,
        };
        match a.checked_mul(b) {
            Some(0) => Wide::Pos(0),
            Some(m) if negative => Wide::Neg(m),
            Some(m) => Wide::Pos(m),
            None => Wide::Overflow,
        }
    }

    /// The two's-complement bits of the low 128.
    fn bits(self) -> u128 {
        match self {
            Wide::Pos(m) => m,
            Wide::Neg(m) => m.wrapping_neg(),
            Wide::Overflow => 0,
        }
    }
}

/// `a op b`, wrapping, for `+ - *` and the bit operations and shifts, and C's truncating `/`
/// and `%`; `//` and `%` with floor (`[TYP-28]`) for signed operands. Divisors are checked
/// already (`DivisionByZero`); a zero one gives zero here rather than faulting the evaluator.
pub fn int_binary(op: BinOp, kind: IntKind, a: &[u8], b: &[u8]) -> Vec<u8> {
    if kind.bits > 128 {
        return wide::binary(op, kind, W256::of(kind, a), W256::of(kind, b)).bytes(kind.bytes());
    }
    kind.encode(int_binary_bits(op, kind, kind.unsigned(a), kind.unsigned(b)))
}

/// `int_binary` on the operands' bits (each within the type's width).
pub fn int_binary_bits(op: BinOp, kind: IntKind, x: u128, y: u128) -> u128 {
    let (sx, sy) = (kind.extend(x), kind.extend(y));
    let result: u128 = match op {
        BinOp::Add => x.wrapping_add(y),
        BinOp::Sub => x.wrapping_sub(y),
        BinOp::Mul => x.wrapping_mul(y),
        BinOp::BitAnd | BinOp::And => x & y,
        BinOp::BitOr | BinOp::Or => x | y,
        BinOp::BitXor => x ^ y,
        BinOp::Shl => {
            let amount = (y as u32).min(127);
            if amount >= kind.bits { 0 } else { x << amount }
        }
        BinOp::Shr => {
            let amount = (y as u32).min(127);
            if kind.signed {
                let v = if amount >= kind.bits { if sx < 0 { -1 } else { 0 } } else { sx >> amount };
                v as u128
            } else if amount >= kind.bits {
                0
            } else {
                x >> amount
            }
        }
        BinOp::Div | BinOp::FloorDiv | BinOp::Rem | BinOp::FloorRem => {
            if y == 0 {
                0
            } else if kind.signed {
                floor_or_trunc(op, sx, sy) as u128
            } else {
                match op {
                    BinOp::Div | BinOp::FloorDiv => x / y,
                    _ => x % y,
                }
            }
        }
        _ => unreachable!("`{}` is not an arithmetic operator", op.c_operator()),
    };
    result & kind.mask()
}

fn floor_or_trunc(op: BinOp, a: i128, b: i128) -> i128 {
    if b == -1 {
        // `MIN / -1` wraps (its check came first); `MIN % -1` is 0.
        return match op {
            BinOp::Div | BinOp::FloorDiv => a.wrapping_neg(),
            _ => 0,
        };
    }
    let q = a / b;
    let r = a % b;
    match op {
        BinOp::Div => q,
        BinOp::Rem => r,
        BinOp::FloorDiv => if r != 0 && ((r < 0) != (b < 0)) { q - 1 } else { q },
        _ => if r != 0 && ((r < 0) != (b < 0)) { r + b } else { r },
    }
}

/// `CheckedBinaryOp`: the result as `int_binary` wraps it, and whether it overflowed: `+ - *`
/// past the type's range, `MIN / -1` and `MIN // -1`.
pub fn int_checked(op: BinOp, kind: IntKind, a: &[u8], b: &[u8]) -> (Vec<u8>, bool) {
    if kind.bits > 128 {
        let (value, overflow) = wide::checked(op, kind, W256::of(kind, a), W256::of(kind, b));
        return (value.bytes(kind.bytes()), overflow);
    }
    let (x, y) = (Wide::of(kind, a), Wide::of(kind, b));
    let exact = match op {
        BinOp::Add => x.add(y),
        BinOp::Sub => x.add(y.neg()),
        BinOp::Mul => x.mul(y),
        BinOp::Div | BinOp::FloorDiv => {
            let overflow = kind.signed && kind.signed_value(a) == kind.min() && kind.signed_value(b) == -1;
            return (int_binary(op, kind, a, b), overflow);
        }
        _ => return (int_binary(op, kind, a, b), false),
    };
    if kind.fits(exact) {
        (kind.encode(exact.bits()), false)
    } else {
        (int_binary(op, kind, a, b), true)
    }
}

pub fn int_compare(op: BinOp, kind: IntKind, a: &[u8], b: &[u8]) -> bool {
    if kind.bits > 128 {
        return wide::compare(op, kind, W256::of(kind, a), W256::of(kind, b));
    }
    int_compare_bits(op, kind, kind.unsigned(a), kind.unsigned(b))
}

pub fn int_compare_bits(op: BinOp, kind: IntKind, x: u128, y: u128) -> bool {
    let ordering = if kind.signed { kind.extend(x).cmp(&kind.extend(y)) } else { x.cmp(&y) };
    compare(op, ordering)
}

/// `int_checked` for a type of at most 64 bits, on the operands' bits: the exact result fits an
/// `i128`.
pub fn int_checked_narrow(op: BinOp, kind: IntKind, x: u128, y: u128) -> (u128, bool) {
    debug_assert!(kind.bits <= 64);
    let (sx, sy) = (kind.extend(x), kind.extend(y));
    match op {
        BinOp::Add | BinOp::Sub | BinOp::Mul if kind.signed => {
            let exact = match op {
                BinOp::Add => sx + sy,
                BinOp::Sub => sx - sy,
                _ => sx * sy,
            };
            (exact as u128 & kind.mask(), exact < kind.min() || exact > kind.max() as i128)
        }
        BinOp::Add | BinOp::Mul => {
            let exact = if op == BinOp::Add { x + y } else { x * y };
            (exact & kind.mask(), exact > kind.max())
        }
        BinOp::Sub => (x.wrapping_sub(y) & kind.mask(), y > x),
        BinOp::Div | BinOp::FloorDiv => {
            let overflow = kind.signed && sx == kind.min() && sy == -1;
            (int_binary_bits(op, kind, x, y), overflow)
        }
        _ => (int_binary_bits(op, kind, x, y), false),
    }
}

/// A comparison of two floats, as C compares them (a NaN is unequal to everything).
pub fn float_compare(op: BinOp, x: f64, y: f64) -> bool {
    match op {
        BinOp::Eq | BinOp::Is => x == y,
        BinOp::Ne | BinOp::IsNot => x != y,
        BinOp::Lt => x < y,
        BinOp::Le => x <= y,
        BinOp::Gt => x > y,
        _ => x >= y,
    }
}

/// Float arithmetic of a `bits`-wide float: an `f32`'s in `f32`, the others' in `f64` (an `f16`
/// rounds once, as it is stored). `None` for an operator floats do not have.
pub fn float_binary(op: BinOp, bits: u32, x: f64, y: f64) -> Option<f64> {
    if bits == 32 {
        let (x, y) = (x as f32, y as f32);
        return Some(
            (match op {
                BinOp::Add => x + y,
                BinOp::Sub => x - y,
                BinOp::Mul => x * y,
                BinOp::Div => x / y,
                BinOp::FloorDiv => floor_div_f32(x, y),
                BinOp::FloorRem => floor_rem_f32(x, y),
                BinOp::Rem => x % y,
                _ => return None,
            }) as f64,
        );
    }
    Some(match op {
        BinOp::Add => x + y,
        BinOp::Sub => x - y,
        BinOp::Mul => x * y,
        BinOp::Div => x / y,
        BinOp::FloorDiv => floor_div_f64(x, y),
        BinOp::FloorRem => floor_rem_f64(x, y),
        BinOp::Rem => x % y,
        _ => return None,
    })
}

pub fn compare(op: BinOp, ordering: std::cmp::Ordering) -> bool {
    use std::cmp::Ordering::*;
    match op {
        BinOp::Eq | BinOp::Is => ordering == Equal,
        BinOp::Ne | BinOp::IsNot => ordering != Equal,
        BinOp::Lt => ordering == Less,
        BinOp::Le => ordering != Greater,
        BinOp::Gt => ordering == Greater,
        BinOp::Ge => ordering != Less,
        _ => unreachable!("`{}` is not a comparison", op.c_operator()),
    }
}

/// The value of an integer as an `f64`, rounded to nearest as C converts.
pub fn int_to_f64(kind: IntKind, bytes: &[u8]) -> f64 {
    if kind.bits > 128 {
        return W256::of(kind, bytes).to_f64(kind.signed);
    }
    if kind.signed { kind.signed_value(bytes) as f64 } else { kind.unsigned(bytes) as f64 }
}

pub fn int_to_f32(kind: IntKind, bytes: &[u8]) -> f32 {
    if kind.bits > 128 {
        return W256::of(kind, bytes).to_f64(kind.signed) as f32;
    }
    if kind.signed { kind.signed_value(bytes) as f32 } else { kind.unsigned(bytes) as f32 }
}

/// `[TYP-6]` — a float to an integer: toward zero, saturating, NaN to 0.
pub fn float_to_int(kind: IntKind, value: f64) -> Vec<u8> {
    if value.is_nan() {
        return kind.encode(0);
    }
    if kind.bits > 128 {
        let (lo, hi) = (W256::min(kind), W256::max(kind));
        let w = if value <= lo.to_f64(kind.signed) {
            lo
        } else if value >= hi.to_f64(kind.signed) {
            hi
        } else {
            W256::from_f64(value)
        };
        return w.bytes(kind.bytes());
    }
    if kind.signed {
        let lo = kind.min() as f64;
        let hi = kind.max() as f64;
        if value <= lo {
            return kind.encode(kind.min() as u128);
        }
        if value >= hi {
            return kind.encode(kind.max());
        }
        kind.encode(value as i128 as u128)
    } else {
        if value <= 0.0 {
            return kind.encode(0);
        }
        if value >= kind.max() as f64 {
            return kind.encode(kind.max());
        }
        kind.encode(value as u128)
    }
}

/// An integer widened or narrowed: truncated, or extended by its sign.
pub fn int_cast(from: IntKind, to: IntKind, bytes: &[u8]) -> Vec<u8> {
    if from.bits > 128 || to.bits > 128 {
        return W256::of(from, bytes).bytes(to.bytes());
    }
    let value = if from.signed { from.signed_value(bytes) as u128 } else { from.unsigned(bytes) };
    to.encode(value)
}

/// `f16` bits as the `f64` that holds them exactly (`ember_f16_to_f64`).
pub fn f16_to_f64(h: u16) -> f64 {
    let sign = ((h & 0x8000) as u64) << 48;
    let exponent = ((h >> 10) & 0x1F) as u64;
    let mantissa = (h & 0x3FF) as u64;
    if exponent == 0x1F {
        return f64::from_bits(sign | 0x7FF0_0000_0000_0000 | (mantissa << 42));
    }
    if exponent != 0 {
        return f64::from_bits(sign | ((exponent + 1008) << 52) | (mantissa << 42));
    }
    let magnitude = mantissa as f64 / 16_777_216.0;
    if sign != 0 { -magnitude } else { magnitude }
}

/// The `f16` nearest `v`, ties to even (`ember_f64_to_f16`), as `ember_types` rounds a literal.
pub fn f64_to_f16(v: f64) -> u16 {
    ember_types::f16_bits(v)
}

/// Python's float `//` and `%` (`[TYP-29]`): `floor(a / b)` and `a - b * floor(a / b)`, each
/// rounded as the machine rounds it.
pub fn floor_div_f64(a: f64, b: f64) -> f64 {
    (a / b).floor()
}

pub fn floor_rem_f64(a: f64, b: f64) -> f64 {
    let q = (a / b).floor();
    a - b * q
}

pub fn floor_div_f32(a: f32, b: f32) -> f32 {
    (a / b).floor()
}

pub fn floor_rem_f32(a: f32, b: f32) -> f32 {
    let q = (a / b).floor();
    a - b * q
}

#[cfg(test)]
mod tests {
    use super::*;

    const I8: IntKind = IntKind { bits: 8, signed: true };
    const U8: IntKind = IntKind { bits: 8, signed: false };
    const I64: IntKind = IntKind { bits: 64, signed: true };

    #[test]
    fn integers_wrap_check_and_floor_as_the_target() {
        assert_eq!(int_binary(BinOp::Add, U8, &[250], &[10]), vec![4]);
        assert_eq!(int_checked(BinOp::Add, I8, &[127], &[1]), (vec![0x80], true));
        assert_eq!(int_checked(BinOp::Sub, I8, &[0x80], &[0]), (vec![0x80], false));
        assert_eq!(int_checked(BinOp::Mul, I8, &[0xF0], &[8]), (vec![0x80], false), "-16 * 8 = -128 fits");
        assert_eq!(int_checked(BinOp::Mul, I8, &[16], &[8]).1, true);
        let neg7 = I64.encode((-7i128) as u128);
        let two = I64.encode(2);
        assert_eq!(I64.signed_value(&int_binary(BinOp::FloorDiv, I64, &neg7, &two)), -4);
        assert_eq!(I64.signed_value(&int_binary(BinOp::FloorRem, I64, &neg7, &two)), 1);
        assert_eq!(I64.signed_value(&int_binary(BinOp::Div, I64, &neg7, &two)), -3);
        assert_eq!(I64.signed_value(&int_binary(BinOp::Shr, I64, &neg7, &I64.encode(1))), -4);
        let min = I64.encode(i64::MIN as i128 as u128);
        let minus_one = I64.encode(u128::MAX);
        assert!(int_checked(BinOp::FloorDiv, I64, &min, &minus_one).1);
        assert_eq!(float_to_int(I8, 1e9), vec![127]);
        assert_eq!(float_to_int(U8, -3.0), vec![0]);
        assert_eq!(float_to_int(I8, f64::NAN), vec![0]);
        assert_eq!(int_cast(I8, I64, &[0xFF]), I64.encode(u128::MAX));
        assert_eq!(f16_to_f64(0x3C00), 1.0);
        assert_eq!(f64_to_f16(1.0), 0x3C00);
        assert_eq!(floor_rem_f64(-1.0, 3.0), 2.0);
    }
}
