//! `[RNG-3a]` — a range type's total operations: `clamped` (two compares, or `fmin`/`fmax` for a
//! float, which make NaN the low end) and `contains`, as the C backend computes them.

use crate::func::{FnCx, Val, float_literal, int_literal};
use crate::types::Sc;
use crate::{Result, unsupported};
use ember_types::{Bound, FloatTy, RangeId, TyKind};

fn endpoint(bound: Bound) -> f64 {
    match bound {
        Bound::Float(v) => v,
        Bound::Int(v) => v as f64,
    }
}

/// The nearest `f64` (or, for `single`, `f32`) after `from` toward `to` (C's `nextafter`).
fn next_after(from: f64, to: f64, single: bool) -> f64 {
    if from.is_nan() || to.is_nan() || from == to {
        return from;
    }
    if single {
        let x = from as f32;
        if x == 0.0 {
            let tiny = f32::from_bits(1);
            return if to > 0.0 { tiny as f64 } else { -tiny as f64 };
        }
        let bits = x.to_bits();
        let up = (to as f32 > x) == (x > 0.0);
        return f32::from_bits(if up { bits + 1 } else { bits - 1 }) as f64;
    }
    if from == 0.0 {
        let tiny = f64::from_bits(1);
        return if to > 0.0 { tiny } else { -tiny };
    }
    let bits = from.to_bits();
    let up = (to > from) == (from > 0.0);
    f64::from_bits(if up { bits + 1 } else { bits - 1 })
}

impl<'a> FnCx<'_, 'a> {
    /// `T.clamped(v)`: `min(max(v, lo), hi)` for an inclusive range, the nearest value strictly
    /// inside a half-open one.
    pub(crate) fn range_clamped(&mut self, id: RangeId, value: Val) -> Result<Val> {
        let def = self.m.ty.table.range_def(id).clone();
        let Val::Imm(sc, value) = value else { return unsupported("a range value held in memory") };
        // D-316 — an `f16` range clamps in `double` between `f16` endpoints.
        if self.m.ty.is_f16(def.repr) {
            let lo = ember_types::f16_value(ember_types::f16_bits(endpoint(def.lo)));
            let bound = endpoint(def.hi);
            let mut hi_bits = ember_types::f16_bits(bound);
            if !def.inclusive && ember_types::f16_value(hi_bits) >= bound {
                hi_bits = match hi_bits {
                    0x0000 | 0x8000 => 0x8001,
                    bits if bits & 0x8000 == 0 => bits - 1,
                    bits => bits + 1,
                };
            }
            let hi = ember_types::f16_value(hi_bits);
            let wide = self.alloca(8, 8);
            self.call_rt("f16_to_f64", vec![Val::Imm(Sc::I(16), value)], Some((&wide, 8)));
            let wide = self.load(Sc::F64, &wide, 8);
            let clamped = self.min_max(&wide, Sc::F64, lo, hi);
            let narrow = self.alloca(2, 2);
            self.call_rt("f64_to_f16", vec![Val::Imm(Sc::F64, clamped)], Some((&narrow, 2)));
            return Ok(Val::Imm(Sc::I(16), self.load(Sc::I(16), &narrow, 2)));
        }
        if matches!(self.m.ty.kind(def.repr), TyKind::Float(_)) {
            let single = matches!(self.m.ty.kind(def.repr), TyKind::Float(FloatTy::F32));
            let lo = endpoint(def.lo);
            let hi = if def.inclusive { endpoint(def.hi) } else { next_after(endpoint(def.hi), lo, single) };
            return Ok(Val::Imm(sc, self.min_max(&value, sc, lo, hi)));
        }
        let Sc::I(bits) = sc else { return unsupported("a range over a non-number") };
        let int = |bound: Bound| match bound {
            Bound::Int(v) => v,
            Bound::Float(v) => v as i128,
        };
        let lo = int_literal(int(def.lo) as u128, bits);
        let hi = int_literal((int(def.hi) - i128::from(!def.inclusive)) as u128, bits);
        let signed = matches!(self.m.ty.kind(def.repr), TyKind::Int(_));
        let (less, more) = if signed { ("slt", "sgt") } else { ("ult", "ugt") };
        let ty = sc.ir();
        let below = self.value(format!("icmp {less} {ty} {value}, {lo}"));
        let above = self.value(format!("icmp {more} {ty} {value}, {hi}"));
        let upper = self.value(format!("select i1 {above}, {ty} {hi}, {ty} {value}"));
        Ok(Val::Imm(sc, self.value(format!("select i1 {below}, {ty} {lo}, {ty} {upper}"))))
    }

    /// `fmin(fmax(v, lo), hi)`: LLVM's `maxnum`/`minnum` are the C library's, NaN giving the other.
    fn min_max(&mut self, value: &str, sc: Sc, lo: f64, hi: f64) -> String {
        let (ty, name) = (sc.ir(), if sc == Sc::F32 { "f32" } else { "f64" });
        let (max, min) = (format!("llvm.maxnum.{name}"), format!("llvm.minnum.{name}"));
        self.m.declare(&max, || format!("declare {ty} @{max}({ty}, {ty})"));
        self.m.declare(&min, || format!("declare {ty} @{min}({ty}, {ty})"));
        let raised = self.value(format!("call {ty} @{max}({ty} {value}, {ty} {})", float_literal(lo, sc)));
        self.value(format!("call {ty} @{min}({ty} {raised}, {ty} {})", float_literal(hi, sc)))
    }

    /// `T.contains(v)`, as the C backend computes it: through the runtime's float test, an
    /// integer converted to `double`.
    pub(crate) fn range_contains(&mut self, id: RangeId, value: Val) -> Result<Val> {
        let def = self.m.ty.table.range_def(id).clone();
        let Val::Imm(sc, value) = value else { return unsupported("a range value held in memory") };
        let inclusive = Val::Imm(Sc::Bool, def.inclusive.to_string());
        let (name, value, lo, hi) = match self.m.ty.kind(def.repr) {
            TyKind::Float(FloatTy::F32) => ("range_contains_f32", value, endpoint(def.lo), endpoint(def.hi)),
            TyKind::Float(FloatTy::F16) => {
                let wide = self.alloca(8, 8);
                self.call_rt("f16_to_f64", vec![Val::Imm(Sc::I(16), value)], Some((&wide, 8)));
                let wide = self.load(Sc::F64, &wide, 8);
                let half = |bound| ember_types::f16_value(ember_types::f16_bits(endpoint(bound)));
                ("range_contains_f64", wide, half(def.lo), half(def.hi))
            }
            TyKind::Float(_) => ("range_contains_f64", value, endpoint(def.lo), endpoint(def.hi)),
            _ => {
                let Sc::I(bits) = sc else { return unsupported("a range over a non-number") };
                let signed = matches!(self.m.ty.kind(def.repr), TyKind::Int(_));
                let convert = if signed { "sitofp" } else { "uitofp" };
                let wide = self.value(format!("{convert} i{bits} {value} to double"));
                ("range_contains_f64", wide, endpoint(def.lo), endpoint(def.hi))
            }
        };
        let float = if name == "range_contains_f32" { Sc::F32 } else { Sc::F64 };
        let slot = self.alloca(1, 1);
        self.call_rt(
            name,
            vec![
                Val::Imm(float, value),
                Val::Imm(float, float_literal(lo, float)),
                Val::Imm(float, float_literal(hi, float)),
                inclusive,
            ],
            Some((&slot, 1)),
        );
        Ok(Val::Imm(Sc::Bool, self.load(Sc::Bool, &slot, 1)))
    }
}

#[cfg(test)]
mod tests {
    use super::next_after;

    #[test]
    fn rng_3a_next_after_matches_c() {
        assert_eq!(next_after(1.0, 0.0, false), 1.0 - f64::EPSILON / 2.0);
        assert_eq!(next_after(0.0, 1.0, false), f64::from_bits(1));
        assert_eq!(next_after(-1.0, 0.0, false), -1.0 + f64::EPSILON / 2.0);
        assert_eq!(next_after(1.0, 0.0, true), (1.0f32 - f32::EPSILON / 2.0) as f64);
    }
}
