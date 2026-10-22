//! `[SIMD-1]`, `[SIMD-4]`, `[SIMD-8]`, `[SIMD-9]` — `std.simd`'s operations lane by lane, as
//! `ember_simd.h` defines them: integer lanes checked (every lane computed, then one panic naming
//! the operator), wrapping or saturating by the policy; float lanes one IEEE operation each;
//! reductions from lane 0 in order; a checked memory operation panicking with the lane whose index
//! is out of bounds. The approximations are exact here, as on a target without SSE.

use ember_hir::SimdOp;
use ember_mir::{BinOp, Operand};
use ember_span::Span;
use ember_types::{OverflowPolicy, SimdLane, SimdShape, Ty, TyKind};

use crate::scalar::{self, IntKind};
use crate::{fault, u64_of, Interp, R};

#[derive(Clone, Copy)]
struct Lanes {
    shape: SimdShape,
    width: usize,
}

impl Lanes {
    fn int(self) -> IntKind {
        IntKind { bits: self.shape.bits, signed: matches!(self.shape.lane, SimdLane::Signed | SimdLane::Mask) }
    }

    fn float(self) -> bool {
        self.shape.lane == SimdLane::Float
    }

    fn count(self) -> usize {
        self.shape.lanes as usize
    }

    fn lane(self, v: &[u8], i: usize) -> &[u8] {
        &v[i * self.width..(i + 1) * self.width]
    }

    fn get_float(self, v: &[u8], i: usize) -> f64 {
        let bytes = self.lane(v, i);
        if self.shape.bits == 32 {
            f64::from(f32::from_bits(u32::from_le_bytes(bytes.try_into().expect("four bytes"))))
        } else {
            f64::from_bits(u64::from_le_bytes(bytes.try_into().expect("eight bytes")))
        }
    }

    fn float_bytes(self, value: f64) -> Vec<u8> {
        if self.shape.bits == 32 { (value as f32).to_bits().to_le_bytes().to_vec() } else { value.to_bits().to_le_bytes().to_vec() }
    }

    fn from_fn(self, mut f: impl FnMut(usize) -> Vec<u8>) -> Vec<u8> {
        (0..self.count()).flat_map(&mut f).collect()
    }
}

/// The saturated result of an overflowing `+ - *` (`EMBER_SIMD_SAT_*`): the bound on the side
/// the true result lies.
fn saturate(op: BinOp, kind: IntKind, x: &[u8], y: &[u8]) -> Vec<u8> {
    let negative = |v: &[u8]| kind.signed && kind.signed_value(v) < 0;
    let low = if kind.signed { kind.encode((-(1i128 << (kind.bits - 1))) as u128) } else { kind.encode(0) };
    let high = if kind.signed { kind.encode((1u128 << (kind.bits - 1)) - 1) } else { kind.encode(u128::MAX) };
    let to_low = match op {
        BinOp::Add => negative(y),
        BinOp::Sub => !kind.signed || !negative(y),
        _ => negative(x) != negative(y),
    };
    if to_low { low } else { high }
}

impl Interp<'_, '_> {
    fn simd_lanes(&self, ty: Ty) -> R<Lanes> {
        let TyKind::Struct(id) = *self.types.kind(ty) else { return fault("a vector that is no struct") };
        let Some(shape) = SimdShape::of(self.types.struct_def(id).name.as_str()) else { return fault("a vector of no `std.simd` type") };
        Ok(Lanes { shape, width: (shape.bits / 8) as usize })
    }

    /// One integer `+ - *` under `policy`, and whether it overflowed.
    fn simd_int(&self, op: BinOp, policy: OverflowPolicy, kind: IntKind, x: &[u8], y: &[u8]) -> (Vec<u8>, bool) {
        let (value, overflowed) = scalar::int_checked(op, kind, x, y);
        match policy {
            OverflowPolicy::Saturate if overflowed => (saturate(op, kind, x, y), false),
            OverflowPolicy::Panic => (value, overflowed),
            _ => (value, false),
        }
    }

    /// A span's `(ptr, len)`.
    fn simd_view(&mut self, operand: &Operand) -> R<(u64, u64)> {
        let view = self.operand(operand, None)?;
        Ok((u64_of(&view[..8]), u64_of(&view[8..16])))
    }

    fn lane_bounds<T>(&mut self, lane: u64, index: u64, len: u64, span: Span) -> R<T> {
        let shown = if index > i64::MAX as u64 { (index as i64).to_string() } else { index.to_string() };
        self.panic(&format!("lane {lane}: index {shown} is out of bounds for a length of {len}"), span)
    }

    /// The lane indices of an index vector, each checked against `len` when `checked`.
    fn simd_indices(&mut self, index: Ty, idx: &[u8], len: u64, checked: bool, span: Span) -> R<Vec<u64>> {
        let lanes = self.simd_lanes(index)?;
        let kind = lanes.int();
        let kind = IntKind { signed: lanes.shape.lane == SimdLane::Signed, ..kind };
        let mut out = Vec::with_capacity(lanes.count());
        for i in 0..lanes.count() {
            let raw = lanes.lane(idx, i);
            let value = if kind.signed { kind.signed_value(raw) as i64 as u64 } else { kind.unsigned(raw) as u64 };
            let negative = kind.signed && (value as i64) < 0;
            if checked && (negative || value >= len) {
                return self.lane_bounds(i as u64, value, len, span);
            }
            out.push(value);
        }
        Ok(out)
    }

    pub(crate) fn simd(&mut self, op: SimdOp, vector: Ty, args: &[Operand], dest_ty: Ty, span: Span) -> R<Option<Vec<u8>>> {
        let lanes = self.simd_lanes(vector)?;
        let (n, width) = (lanes.count(), lanes.width);
        let arg = |interp: &mut Self, i: usize| interp.operand(&args[i], None);
        let overflow = |interp: &mut Self, symbol: &str| interp.panic::<Option<Vec<u8>>>(&format!("integer overflow in `{symbol}`"), span);
        Ok(Some(match op {
            SimdOp::Splat => {
                let x = self.operand(&args[0], None)?;
                lanes.from_fn(|_| x[..width].to_vec())
            }
            SimdOp::MaskSplat => {
                let set = self.operand(&args[0], None)?[0] != 0;
                lanes.from_fn(|_| vec![if set { 0xFF } else { 0 }; width])
            }
            SimdOp::Load { checked } => {
                let (ptr, len) = self.simd_view(&args[0])?;
                if checked && len < n as u64 {
                    return self.lane_bounds(len, len, len, span);
                }
                self.mem.read(ptr, (n * width) as u64)?.to_vec()
            }
            SimdOp::Store { checked } => {
                let v = arg(self, 0)?;
                let (ptr, len) = self.simd_view(&args[1])?;
                if checked && len < n as u64 {
                    return self.lane_bounds(len, len, len, span);
                }
                self.mem.write(ptr, &v[..n * width])?;
                return Ok(None);
            }
            SimdOp::Gather { checked, index } => {
                let (ptr, len) = self.simd_view(&args[0])?;
                let idx = arg(self, 1)?;
                let at = self.simd_indices(index, &idx, len, checked, span)?;
                let mut out = Vec::with_capacity(n * width);
                for i in at {
                    out.extend_from_slice(self.mem.read(ptr.wrapping_add(i.wrapping_mul(width as u64)), width as u64)?);
                }
                out
            }
            SimdOp::Scatter { checked, index } => {
                let v = arg(self, 0)?;
                let (ptr, len) = self.simd_view(&args[1])?;
                let idx = arg(self, 2)?;
                let at = self.simd_indices(index, &idx, len, checked, span)?;
                for (lane, i) in at.into_iter().enumerate() {
                    self.mem.write(ptr.wrapping_add(i.wrapping_mul(width as u64)), lanes.lane(&v, lane))?;
                }
                return Ok(None);
            }
            SimdOp::Binary { op: binary, policy } => {
                let (a, b) = (arg(self, 0)?, arg(self, 1)?);
                if lanes.float() && matches!(binary, BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div) {
                    let bits = lanes.shape.bits;
                    lanes.from_fn(|i| {
                        let value = scalar::float_binary(binary, bits, lanes.get_float(&a, i), lanes.get_float(&b, i)).unwrap_or(f64::NAN);
                        lanes.float_bytes(value)
                    })
                } else if matches!(binary, BinOp::Add | BinOp::Sub | BinOp::Mul) {
                    let kind = IntKind { signed: lanes.shape.lane == SimdLane::Signed, ..lanes.int() };
                    let mut any = false;
                    let out = lanes.from_fn(|i| {
                        let (value, overflowed) = self.simd_int(binary, policy, kind, lanes.lane(&a, i), lanes.lane(&b, i));
                        any |= overflowed;
                        value
                    });
                    if any {
                        return overflow(self, binary.c_operator());
                    }
                    out
                } else {
                    let f = |x: u8, y: u8| match binary {
                        BinOp::BitAnd => x & y,
                        BinOp::BitOr => x | y,
                        _ => x ^ y,
                    };
                    a.iter().zip(&b).take(n * width).map(|(x, y)| f(*x, *y)).collect()
                }
            }
            SimdOp::Neg { policy } => {
                let a = arg(self, 0)?;
                if lanes.float() {
                    lanes.from_fn(|i| lanes.float_bytes(-lanes.get_float(&a, i)))
                } else {
                    let kind = IntKind { signed: lanes.shape.lane == SimdLane::Signed, ..lanes.int() };
                    let zero = vec![0u8; width];
                    let mut any = false;
                    let out = lanes.from_fn(|i| {
                        let (value, overflowed) = self.simd_int(BinOp::Sub, policy, kind, &zero, lanes.lane(&a, i));
                        any |= overflowed;
                        value
                    });
                    if any {
                        return overflow(self, "-");
                    }
                    out
                }
            }
            SimdOp::Abs { policy } => {
                let a = arg(self, 0)?;
                if lanes.float() {
                    lanes.from_fn(|i| {
                        let x = lanes.get_float(&a, i);
                        lanes.float_bytes(if x < 0.0 { -x } else { x + 0.0 })
                    })
                } else {
                    let kind = IntKind { signed: lanes.shape.lane == SimdLane::Signed, ..lanes.int() };
                    let zero = vec![0u8; width];
                    let mut any = false;
                    let out = lanes.from_fn(|i| {
                        let x = lanes.lane(&a, i);
                        if !kind.signed || kind.signed_value(x) >= 0 {
                            return x.to_vec();
                        }
                        let (value, overflowed) = self.simd_int(BinOp::Sub, policy, kind, &zero, x);
                        any |= overflowed;
                        // `max(a, -a)`: a wrapped `MIN` stays `MIN`.
                        if kind.signed_value(&value) < kind.signed_value(x) { x.to_vec() } else { value }
                    });
                    if any {
                        return overflow(self, "-");
                    }
                    out
                }
            }
            SimdOp::Compare { op: compare } => {
                let (a, b) = (arg(self, 0)?, arg(self, 1)?);
                let mask = self.simd_lanes(dest_ty)?;
                let kind = IntKind { signed: lanes.shape.lane == SimdLane::Signed, ..lanes.int() };
                mask.from_fn(|i| {
                    let holds = if lanes.float() {
                        scalar::float_compare(compare, lanes.get_float(&a, i), lanes.get_float(&b, i))
                    } else {
                        scalar::int_compare(compare, kind, lanes.lane(&a, i), lanes.lane(&b, i))
                    };
                    vec![if holds { 0xFF } else { 0 }; mask.width]
                })
            }
            SimdOp::ReduceAdd { policy } => {
                let v = arg(self, 0)?;
                if lanes.float() {
                    let bits = lanes.shape.bits;
                    let mut r = lanes.get_float(&v, 0);
                    for i in 1..n {
                        r = scalar::float_binary(BinOp::Add, bits, r, lanes.get_float(&v, i)).unwrap_or(f64::NAN);
                    }
                    lanes.float_bytes(r)
                } else {
                    let kind = IntKind { signed: lanes.shape.lane == SimdLane::Signed, ..lanes.int() };
                    let mut r = lanes.lane(&v, 0).to_vec();
                    for i in 1..n {
                        let (value, overflowed) = self.simd_int(BinOp::Add, policy, kind, &r, lanes.lane(&v, i));
                        if overflowed {
                            return overflow(self, "+");
                        }
                        r = value;
                    }
                    r
                }
            }
            SimdOp::ReduceMin | SimdOp::ReduceMax => {
                let v = arg(self, 0)?;
                let kind = IntKind { signed: lanes.shape.lane == SimdLane::Signed, ..lanes.int() };
                let better = if op == SimdOp::ReduceMin { BinOp::Lt } else { BinOp::Gt };
                let mut r = lanes.lane(&v, 0).to_vec();
                for i in 1..n {
                    let x = lanes.lane(&v, i);
                    let wins = if lanes.float() {
                        let at = |bytes: &[u8]| lanes.get_float(bytes, 0);
                        scalar::float_compare(better, at(x), at(&r))
                    } else {
                        scalar::int_compare(better, kind, x, &r)
                    };
                    if wins {
                        r = x.to_vec();
                    }
                }
                r
            }
            SimdOp::Min | SimdOp::Max => {
                let (a, b) = (arg(self, 0)?, arg(self, 1)?);
                let kind = IntKind { signed: lanes.shape.lane == SimdLane::Signed, ..lanes.int() };
                let first = if op == SimdOp::Min { BinOp::Lt } else { BinOp::Gt };
                lanes.from_fn(|i| {
                    let (x, y) = (lanes.lane(&a, i), lanes.lane(&b, i));
                    let take_first = if lanes.float() {
                        scalar::float_compare(first, lanes.get_float(x, 0), lanes.get_float(y, 0))
                    } else {
                        scalar::int_compare(first, kind, x, y)
                    };
                    if take_first { x.to_vec() } else { y.to_vec() }
                })
            }
            SimdOp::Shuffle => {
                let v = arg(self, 0)?;
                let order = arg(self, 1)?;
                lanes.from_fn(|i| {
                    let from = u64_of(&order[8 * i..8 * i + 8]) as usize;
                    lanes.lane(&v, from.min(n - 1)).to_vec()
                })
            }
            SimdOp::Fma => {
                let (a, b, c) = (arg(self, 0)?, arg(self, 1)?, arg(self, 2)?);
                lanes.from_fn(|i| {
                    let (x, y, z) = (lanes.get_float(&a, i), lanes.get_float(&b, i), lanes.get_float(&c, i));
                    if lanes.shape.bits == 32 {
                        lanes.float_bytes(f64::from((x as f32).mul_add(y as f32, z as f32)))
                    } else {
                        lanes.float_bytes(x.mul_add(y, z))
                    }
                })
            }
            SimdOp::Sqrt | SimdOp::RsqrtApprox | SimdOp::RcpApprox => {
                let a = arg(self, 0)?;
                lanes.from_fn(|i| {
                    let x = lanes.get_float(&a, i);
                    let value = if lanes.shape.bits == 32 {
                        let x = x as f32;
                        f64::from(match op {
                            SimdOp::Sqrt => x.sqrt(),
                            SimdOp::RsqrtApprox => 1.0 / x.sqrt(),
                            _ => 1.0 / x,
                        })
                    } else {
                        match op {
                            SimdOp::Sqrt => x.sqrt(),
                            SimdOp::RsqrtApprox => 1.0 / x.sqrt(),
                            _ => 1.0 / x,
                        }
                    };
                    lanes.float_bytes(value)
                })
            }
            SimdOp::Any | SimdOp::All => {
                let m = arg(self, 0)?;
                let set = (0..n).map(|i| lanes.lane(&m, i).iter().any(|b| *b != 0));
                let result = if op == SimdOp::Any { set.into_iter().any(|s| s) } else { set.into_iter().all(|s| s) };
                vec![u8::from(result)]
            }
            SimdOp::Select => {
                let m = arg(self, 0)?;
                let (a, b) = (arg(self, 1)?, arg(self, 2)?);
                let mask = Lanes { width: m.len() / n, ..lanes };
                lanes.from_fn(|i| {
                    let set = mask.lane(&m, i).iter().any(|b| *b != 0);
                    if set { lanes.lane(&a, i).to_vec() } else { lanes.lane(&b, i).to_vec() }
                })
            }
        }))
    }
}
