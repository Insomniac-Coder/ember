#$ test: run-pass
#$ rules: SIMD-8
#$ profiles: debug, release, shipping
#$ stdout: [11, 22, 33, 44] [-9, -18, -27, -36] [10, 40, 90, 160] [-1, -2, -3, -4]
#$ [0, 4, 0, 4] [1, 6, 3, 6] [1, 2, 3, 2]
#$ [-294967296, 5, 5, 5] [-294967296, 5, 5, 5]
#$ [155, 0, 245, 1] [127, -128, 127, -128]
#$ [true, false, false, true] [false, true, true, false]
#$ true true
#$ [0.5, 0.25] [-12.0, 2.0]
# `[SIMD-8]` — vector operators are lane-wise and follow the scalar rules: integer lanes wrap only
# where the function is `@overflow(wrap)` (or saturate under `@overflow(saturate)`) or the code
# calls `wrapping_add` and friends, and float lanes are IEEE with no contraction: `a * b + c` is
# two roundings, not `fma`'s one.
from std.simd import i32x4, u8x16, i8x16, f32x4, f64x2

@overflow(wrap)
fn wrapped(a: i32x4, b: i32x4) -> i32x4:
    return a + b

@overflow(saturate)
fn saturated(a: u8x16, b: u8x16) -> u8x16:
    return a + b - b

@overflow(saturate)
fn saturated_signed(a: i8x16, b: i8x16) -> i8x16:
    return a * b

fn first4(v: u8x16) -> Array[int]:
    out: Array[int] = []
    for i in 0..4:
        out.push(v.to_array()[i] as int)
    return out

fn first4_i8(v: i8x16) -> Array[int]:
    out: Array[int] = []
    for i in 0..4:
        out.push(v.to_array()[i] as int)
    return out

fn main():
    a = i32x4.from_array([1, 2, 3, 4])
    b = i32x4.splat(10)
    println((a + b * a).to_array(), (a - b * a).to_array(), (a * b * a).to_array(), (-a).to_array())
    c = i32x4.from_array([1, 6, 3, 6])
    println((c & i32x4.splat(4)).to_array(), (c | a).to_array(), (c ^ i32x4.from_array([0, 4, 0, 4])).to_array())
    big = i32x4.from_array([2_000_000_000, 2, 2, 2])
    three = i32x4.from_array([2_000_000_000, 3, 3, 3])
    println(wrapped(big, three).to_array(), big.wrapping_add(three).to_array())
    hi = u8x16.from_array([200, 0, 250, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])
    lo = u8x16.from_array([100, 0, 10, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])
    s = i8x16.from_array([100, -100, -64, 64, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])
    t = i8x16.from_array([2, 2, -2, -2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])
    println(first4(saturated(hi, lo)), first4_i8(saturated_signed(s, t)))
    println((a == i32x4.from_array([1, 0, 0, 4])).to_array(), (a != i32x4.from_array([1, 0, 0, 4])).to_array())
    # 1 + 2^-12 squared is 1 + 2^-11 + 2^-24: `*` rounds it to 1 + 2^-11 before `+` takes 1
    # away, where `fma` rounds once, after, and keeps the 2^-24.
    e: f32 = 1.0 / 4096.0
    x = f32x4.splat(1.0 + e)
    minus_one = f32x4.splat(-1.0)
    unfused = (x * x + minus_one).to_array()[0]
    fused = x.fma(x, minus_one).to_array()[0]
    println(unfused == 2.0 * e, fused == 2.0 * e + e * e)
    d = f64x2.from_array([1.0, -8.0])
    println((d / f64x2.from_array([2.0, -4.0]) * f64x2.from_array([1.0, 0.125])).to_array(), (d * f64x2.splat(-4.0) - f64x2.from_array([8.0, 30.0])).to_array())
