//! `[TYP-1]` — `i256` and `u256`, as the runtime's `ember_i256`/`ember_u256` hold them: four
//! 64-bit words, least significant first, two's complement for the signed one. Every operation
//! the narrower integers have is here on 256 bits, wrapping, with the checked forms reporting an
//! overflow as `int_checked` does.

use std::cmp::Ordering;

use ember_mir::BinOp;

use crate::scalar::IntKind;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct W256(pub [u64; 4]);

impl W256 {
    pub const ZERO: W256 = W256([0; 4]);

    /// The value of `bytes` (little-endian, `kind`'s width), extended to 256 bits by its sign.
    pub fn of(kind: IntKind, bytes: &[u8]) -> W256 {
        let n = kind.bytes().min(bytes.len()).min(32);
        let negative = kind.signed && n > 0 && bytes[n - 1] & 0x80 != 0;
        let mut wide = [if negative { 0xFF } else { 0 }; 32];
        wide[..n].copy_from_slice(&bytes[..n]);
        W256::from_le(&wide)
    }

    pub fn from_le(bytes: &[u8; 32]) -> W256 {
        let mut words = [0u64; 4];
        for (i, word) in words.iter_mut().enumerate() {
            *word = u64::from_le_bytes(bytes[8 * i..8 * i + 8].try_into().expect("eight bytes"));
        }
        W256(words)
    }

    pub fn from_u128(v: u128) -> W256 {
        W256([v as u64, (v >> 64) as u64, 0, 0])
    }

    #[cfg(test)]
    pub fn from_i128(v: i128) -> W256 {
        let fill = if v < 0 { u64::MAX } else { 0 };
        W256([v as u64, ((v as u128) >> 64) as u64, fill, fill])
    }

    /// The low `bytes` bytes, little-endian.
    pub fn bytes(self, bytes: usize) -> Vec<u8> {
        let mut out: Vec<u8> = self.0.iter().flat_map(|w| w.to_le_bytes()).collect();
        out.truncate(bytes);
        out
    }

    pub fn low_u128(self) -> u128 {
        u128::from(self.0[0]) | (u128::from(self.0[1]) << 64)
    }

    pub fn negative(self) -> bool {
        self.0[3] >> 63 != 0
    }

    pub fn is_zero(self) -> bool {
        self.0 == [0; 4]
    }

    pub fn add(self, other: W256) -> W256 {
        let mut out = [0u64; 4];
        let mut carry = false;
        for i in 0..4 {
            let (s, c1) = self.0[i].overflowing_add(other.0[i]);
            let (s, c2) = s.overflowing_add(u64::from(carry));
            out[i] = s;
            carry = c1 || c2;
        }
        W256(out)
    }

    pub fn not(self) -> W256 {
        W256(self.0.map(|w| !w))
    }

    pub fn neg(self) -> W256 {
        self.not().add(W256::from_u128(1))
    }

    pub fn sub(self, other: W256) -> W256 {
        self.add(other.neg())
    }

    /// The full 512-bit product of two magnitudes, least significant word first.
    fn product(self, other: W256) -> [u64; 8] {
        let mut out = [0u64; 8];
        for i in 0..4 {
            let mut carry = 0u128;
            for j in 0..4 {
                let t = u128::from(self.0[i]) * u128::from(other.0[j]) + u128::from(out[i + j]) + carry;
                out[i + j] = t as u64;
                carry = t >> 64;
            }
            out[i + 4] = carry as u64;
        }
        out
    }

    pub fn mul(self, other: W256) -> W256 {
        let p = self.product(other);
        W256([p[0], p[1], p[2], p[3]])
    }

    pub fn bitwise(self, other: W256, f: impl Fn(u64, u64) -> u64) -> W256 {
        W256([f(self.0[0], other.0[0]), f(self.0[1], other.0[1]), f(self.0[2], other.0[2]), f(self.0[3], other.0[3])])
    }

    pub fn shl(self, amount: u32) -> W256 {
        if amount >= 256 {
            return W256::ZERO;
        }
        let (words, bits) = ((amount / 64) as usize, amount % 64);
        let mut out = [0u64; 4];
        for i in (words..4).rev() {
            let mut w = self.0[i - words] << bits;
            if bits > 0 && i > words {
                w |= self.0[i - words - 1] >> (64 - bits);
            }
            out[i] = w;
        }
        W256(out)
    }

    /// A right shift, filling with the sign when `arithmetic`.
    pub fn shr(self, amount: u32, arithmetic: bool) -> W256 {
        let fill = if arithmetic && self.negative() { u64::MAX } else { 0 };
        if amount >= 256 {
            return W256([fill; 4]);
        }
        let (words, bits) = ((amount / 64) as usize, amount % 64);
        let word = |i: usize| if i < 4 { self.0[i] } else { fill };
        let mut out = [0u64; 4];
        for (i, slot) in out.iter_mut().enumerate() {
            let mut w = word(i + words) >> bits;
            if bits > 0 {
                w |= word(i + words + 1) << (64 - bits);
            }
            *slot = w;
        }
        W256(out)
    }

    pub fn cmp_unsigned(self, other: W256) -> Ordering {
        self.0.iter().rev().cmp(other.0.iter().rev())
    }

    pub fn cmp_signed(self, other: W256) -> Ordering {
        match (self.negative(), other.negative()) {
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            _ => self.cmp_unsigned(other),
        }
    }

    fn bit(self, i: u32) -> bool {
        (self.0[(i / 64) as usize] >> (i % 64)) & 1 != 0
    }

    /// Unsigned division: the quotient and the remainder (the divisor nonzero).
    pub fn div_rem(self, divisor: W256) -> (W256, W256) {
        let (mut quotient, mut remainder) = (W256::ZERO, W256::ZERO);
        for i in (0..256).rev() {
            remainder = remainder.shl(1);
            if self.bit(i) {
                remainder.0[0] |= 1;
            }
            if remainder.cmp_unsigned(divisor) != Ordering::Less {
                remainder = remainder.sub(divisor);
                quotient.0[(i / 64) as usize] |= 1 << (i % 64);
            }
        }
        (quotient, remainder)
    }

    pub fn abs(self) -> W256 {
        if self.negative() { self.neg() } else { self }
    }

    /// The digits of the magnitude in `base`, most significant first.
    pub fn digits(self, base: u32) -> String {
        if self.is_zero() {
            return "0".to_string();
        }
        let mut out = Vec::new();
        let mut value = self;
        let divisor = W256::from_u128(u128::from(base));
        while !value.is_zero() {
            let (q, r) = value.div_rem(divisor);
            out.push(char::from_digit(r.0[0] as u32, base).expect("a digit"));
            value = q;
        }
        out.iter().rev().collect()
    }

    pub fn to_f64(self, signed: bool) -> f64 {
        let negative = signed && self.negative();
        let m = if negative { self.neg() } else { self };
        let value = m.0.iter().rev().fold(0.0f64, |acc, &w| acc * 18_446_744_073_709_551_616.0 + w as f64);
        if negative { -value } else { value }
    }

    pub fn from_f64(value: f64) -> W256 {
        let negative = value < 0.0;
        let mut m = value.abs().trunc();
        let mut words = [0u64; 4];
        for word in words.iter_mut() {
            *word = (m % 18_446_744_073_709_551_616.0) as u64;
            m = (m / 18_446_744_073_709_551_616.0).trunc();
        }
        let w = W256(words);
        if negative { w.neg() } else { w }
    }

    /// `kind`'s least and greatest values (`kind` 256 bits wide).
    pub fn min(kind: IntKind) -> W256 {
        if kind.signed { W256([0, 0, 0, 1 << 63]) } else { W256::ZERO }
    }

    pub fn max(kind: IntKind) -> W256 {
        if kind.signed { W256([u64::MAX, u64::MAX, u64::MAX, u64::MAX >> 1]) } else { W256([u64::MAX; 4]) }
    }

    /// A decimal text's value, if it fits `kind` (`[TXT-10]`: digits and an optional sign).
    pub fn parse(kind: IntKind, text: &[u8]) -> Option<W256> {
        let (negative, digits) = match text.first() {
            Some(b'-') => (true, &text[1..]),
            Some(b'+') => (false, &text[1..]),
            _ => (false, text),
        };
        if digits.is_empty() || !digits.iter().all(u8::is_ascii_digit) || (negative && !kind.signed) {
            return None;
        }
        let ten = W256::from_u128(10);
        let limit = if negative { W256::max(kind).add(W256::from_u128(1)) } else { W256::max(kind) };
        let mut value = W256::ZERO;
        for d in digits {
            let p = value.product(ten);
            if p[4..].iter().any(|w| *w != 0) {
                return None;
            }
            let product = W256([p[0], p[1], p[2], p[3]]);
            value = product.add(W256::from_u128(u128::from(d - b'0')));
            if value.cmp_unsigned(product) == Ordering::Less || value.cmp_unsigned(limit) == Ordering::Greater {
                return None;
            }
        }
        Some(if negative { value.neg() } else { value })
    }
}

/// `int_binary` on 256 bits: wrapping, Python's floor division for `//` and `%%`, a zero divisor
/// giving zero (its check came first).
pub fn binary(op: BinOp, kind: IntKind, a: W256, b: W256) -> W256 {
    match op {
        BinOp::Add => a.add(b),
        BinOp::Sub => a.sub(b),
        BinOp::Mul => a.mul(b),
        BinOp::BitAnd | BinOp::And => a.bitwise(b, |x, y| x & y),
        BinOp::BitOr | BinOp::Or => a.bitwise(b, |x, y| x | y),
        BinOp::BitXor => a.bitwise(b, |x, y| x ^ y),
        BinOp::Shl => a.shl(b.low_u128().min(256) as u32),
        BinOp::Shr => a.shr(b.low_u128().min(256) as u32, kind.signed),
        BinOp::Div | BinOp::FloorDiv | BinOp::Rem | BinOp::FloorRem => {
            if b.is_zero() {
                return W256::ZERO;
            }
            if !kind.signed {
                let (q, r) = a.div_rem(b);
                return if matches!(op, BinOp::Div | BinOp::FloorDiv) { q } else { r };
            }
            let (q, r) = a.abs().div_rem(b.abs());
            let q = if a.negative() != b.negative() { q.neg() } else { q };
            let r = if a.negative() { r.neg() } else { r };
            let adjust = !r.is_zero() && r.negative() != b.negative();
            match op {
                BinOp::Div => q,
                BinOp::Rem => r,
                BinOp::FloorDiv => if adjust { q.sub(W256::from_u128(1)) } else { q },
                _ => if adjust { r.add(b) } else { r },
            }
        }
        _ => unreachable!("`{}` is not an arithmetic operator", op.c_operator()),
    }
}

/// `CheckedBinaryOp` on 256 bits: the wrapped result and whether it overflowed.
pub fn checked(op: BinOp, kind: IntKind, a: W256, b: W256) -> (W256, bool) {
    let result = binary(op, kind, a, b);
    let overflow = match op {
        BinOp::Add if kind.signed => a.negative() == b.negative() && result.negative() != a.negative(),
        BinOp::Add => result.cmp_unsigned(a) == Ordering::Less,
        BinOp::Sub if kind.signed => a.negative() != b.negative() && result.negative() != a.negative(),
        BinOp::Sub => a.cmp_unsigned(b) == Ordering::Less,
        BinOp::Mul => {
            let p = a.abs().product(b.abs());
            let high = p[4..].iter().any(|w| *w != 0);
            if !kind.signed {
                high
            } else {
                let magnitude = W256([p[0], p[1], p[2], p[3]]);
                let negative = !magnitude.is_zero() && a.negative() != b.negative();
                let limit = if negative { W256::max(kind).add(W256::from_u128(1)) } else { W256::max(kind) };
                high || magnitude.cmp_unsigned(limit) == Ordering::Greater
            }
        }
        BinOp::Div | BinOp::FloorDiv => kind.signed && a == W256::min(kind) && b == W256([u64::MAX; 4]),
        _ => false,
    };
    (result, overflow)
}

pub fn compare(op: BinOp, kind: IntKind, a: W256, b: W256) -> bool {
    crate::scalar::compare(op, if kind.signed { a.cmp_signed(b) } else { a.cmp_unsigned(b) })
}

#[cfg(test)]
mod tests {
    use super::*;

    const I256: IntKind = IntKind { bits: 256, signed: true };
    const U256: IntKind = IntKind { bits: 256, signed: false };

    #[test]
    fn two_hundred_fifty_six_bits_add_multiply_divide_and_print() {
        let big = W256::parse(U256, b"115792089237316195423570985008687907853269984665640564039457584007913129639935").unwrap();
        assert_eq!(big, W256::max(U256));
        assert_eq!(checked(BinOp::Add, U256, big, W256::from_u128(1)), (W256::ZERO, true));
        let x = W256::from_i128(-7);
        let y = W256::from_i128(2);
        assert_eq!(binary(BinOp::FloorDiv, I256, x, y), W256::from_i128(-4));
        assert_eq!(binary(BinOp::FloorRem, I256, x, y), W256::from_i128(1));
        assert_eq!(binary(BinOp::Div, I256, x, y), W256::from_i128(-3));
        assert_eq!(binary(BinOp::Rem, I256, x, y), W256::from_i128(-1));
        let p = W256::from_u128(u128::MAX).mul(W256::from_u128(u128::MAX));
        assert_eq!(p.digits(10), "115792089237316195423570985008687907852589419931798687112530834793049593217025");
        assert_eq!(W256::from_u128(255).shl(250).shr(250, false), W256::from_u128(63));
        assert_eq!(W256::from_i128(-1).shr(255, true), W256::from_i128(-1));
        assert!(checked(BinOp::Mul, I256, W256::max(I256), W256::from_i128(2)).1);
        assert!(!checked(BinOp::Mul, I256, W256::min(I256), W256::from_i128(1)).1);
        assert_eq!(W256::parse(I256, b"-57896044618658097711785492504343953926634992332820282019728792003956564819968"), Some(W256::min(I256)));
        assert_eq!(W256::parse(I256, b"57896044618658097711785492504343953926634992332820282019728792003956564819968"), None);
    }
}
