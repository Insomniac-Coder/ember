#$ test: run-pass
#$ rules: SIMD-1
#$ profiles: debug, development
#$ stdout: [0.0, 2.0, 4.0, 6.0, 8.0, 10.0, 12.0, 14.0, 16.0, 18.0, 20.0, 22.0, 24.0, 26.0, 28.0, 30.0]
#$ 8 4 2 16 32 64
#$ 4 2 16 8
#$ 32 32 64 16 16
#$ 4.0 16.0 4.0 8.0
#$ 32 64 8 16 32
#$ 8 16 4 8
#$ 32 64 8 16 32
#$ 4 8 16 2 4 8
#$ [true, true, false, false]
# `[SIMD-1]` — `std.simd` provides `f32x4`, `f32x8`, `f32x16`, `f64x2`, `f64x4`, `f64x8`, the
# signed and unsigned integer vectors of 8 to 64-bit lanes, and mask types; they are `Copy` value
# types aligned to their size; `V.LANES` is the lane count and `simd.native_lanes[T]()` the
# target's preferred one (16-byte registers on every target of the C backend). Part XII.2's own
# `scale` runs first.
from std.simd import f32x4, f32x8, f32x16, f64x2, f64x4, f64x8, i8x16, i8x32, i8x64, i16x8, i16x16, i16x32, i32x4, i32x8, i32x16, i64x2, i64x4, i64x8
from std.simd import u8x16, u8x32, u8x64, u16x8, u16x16, u16x32, u32x4, u32x8, u32x16, u64x2, u64x4, u64x8, mask32x4
import std.simd

fn scale(xs: Span[f32], mut out: MutSpan[f32], k: f32):
    assert(xs.len() == out.len() and xs.len() % 8 == 0)
    kv = f32x8.splat(k)
    for i in (0..xs.len()).step_by(8):
        v = f32x8.load(xs[i..i + 8])
        (v * kv).store(out[i..i + 8])

fn main():
    xs: Array[f32] = []
    out: Array[f32] = []
    for i in 0..16:
        xs.push(i as f32)
        out.push(0.0)
    scale(xs, out, 2.0)
    println(out)
    println(f32x8.LANES, f32x4.LANES, f64x2.LANES, i8x16.LANES, i8x32.LANES, u8x64.LANES)
    println(simd.native_lanes[f32](), simd.native_lanes[f64](), simd.native_lanes[u8](), simd.native_lanes[i16]())
    # Aligned to their size: a copy is a plain value, the original unchanged.
    a = f32x8.splat(1.0)
    b = a
    println(size_of[f32x8](), align_of[f32x8](), align_of[f64x8](), align_of[i32x4](), align_of[mask32x4]())
    println(f32x4.splat(1.0).reduce_add(), f32x16.splat(1.0).reduce_add(), f64x4.splat(1.0).reduce_add(), f64x8.splat(1.0).reduce_add() + (b.reduce_add() - a.reduce_add()) as f64)
    println(i8x32.splat(1).reduce_add(), i8x64.splat(1).reduce_add(), i16x8.splat(1).reduce_add(), i16x16.splat(1).reduce_add(), i16x32.splat(1).reduce_add())
    println(i32x8.splat(1).reduce_add(), i32x16.splat(1).reduce_add(), i64x4.splat(1).reduce_add(), i64x8.splat(1).reduce_add())
    println(u8x32.splat(1).reduce_add(), u8x64.splat(1).reduce_add(), u16x8.splat(1).reduce_add(), u16x16.splat(1).reduce_add(), u16x32.splat(1).reduce_add())
    println(u32x4.splat(1).reduce_add(), u32x8.splat(1).reduce_add(), u32x16.splat(1).reduce_add(), u64x2.splat(1).reduce_add(), u64x4.splat(1).reduce_add(), u64x8.splat(1).reduce_add())
    m: mask32x4 = i32x4.from_array([1, 2, 3, 4]) < i32x4.splat(3)
    println(m.to_array())
