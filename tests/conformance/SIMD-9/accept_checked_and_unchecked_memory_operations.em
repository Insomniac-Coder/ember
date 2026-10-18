#$ test: run-pass
#$ rules: SIMD-9
#$ profiles: debug, release
#$ stdout: [1.0, 2.0, 3.0, 4.0]
#$ [0.0, 2.0, 4.0, 6.0, 8.0, 0.0]
#$ [30, 10, 40, 20]
#$ [7, 0, 0, 7] [9, 8, 0, 0]
# `[SIMD-9]` — `V.load(s)` and `v.store(s)` take any view of at least `V.LANES` elements;
# `V.gather(s, idx)` and `v.scatter(s, idx)` read and write the element each lane's index names.
# The unchecked forms do the same with no check, in `unsafe`.
from std.simd import f32x4, i64x4, u32x4

fn shifted(xs: Span[f32], mut out: MutSpan[f32]):
    (f32x4.load(xs[1..]) - f32x4.splat(2.0) + f32x4.load(xs[1..5])).store(out[1..])

fn main():
    xs: Array[f32] = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0]
    v = f32x4.load(xs)
    println(v.to_array())
    out: Array[f32] = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0]
    shifted(xs, out)
    println(out)
    ints: Array[i64] = [10, 20, 30, 40]
    g: i64x4 = i64x4.splat(0)
    unsafe:  # SAFETY: every index is below the length of `ints`
        g = i64x4.gather_unchecked(ints, u32x4.from_array([2, 0, 3, 1]))
    println(g.to_array())
    target: Array[i64] = [0, 0, 0, 0]
    i64x4.splat(7).scatter(target, i64x4.from_array([0, 3, 3, 0]))
    other: Array[i64] = [0, 0, 0, 0]
    unsafe:  # SAFETY: every index is below the length of `other`
        i64x4.from_array([9, 8, 9, 8]).scatter_unchecked(other, u32x4.from_array([0, 1, 0, 1]))
    println(target, other)
    loaded: f32x4 = f32x4.splat(0.0)
    unsafe:  # SAFETY: `xs` and `out` hold at least four elements each
        loaded = f32x4.load_unchecked(xs)
        loaded.store_unchecked(out)
