#$ test: run-pass
#$ rules: SIMD-4
#$ profiles: debug, release
#$ stdout: 10 -3 7
#$ 10.5 -2.5 6.0
#$ [4, 3, 2, 1] [1, 1, 4, 4]
#$ [13.0, 10.0, 11.0, 13.0]
#$ [0.0, 13.0, 0.0, 11.0]
#$ [7.0, 17.0]
#$ true true
#$ [1, 2, 10, 10]
# `[SIMD-4]` — horizontal operations (`reduce_add`, `reduce_min`, `reduce_max`), shuffles with a
# constant lane list, gathers, scatters, `fma`, and the approximations `rsqrt_approx` and
# `rcp_approx` are provided. An approximation's last bits are the target's, so it is compared
# with the exact value within its stated error rather than printed.
from std.simd import i32x4, f32x4, f64x2, i64x4
import std.math

fn main():
    v = i32x4.from_array([7, -3, 2, 4])
    println(v.reduce_add(), v.reduce_min(), v.reduce_max())
    f = f32x4.from_array([1.5, -2.5, 6.0, 5.5])
    println(f.reduce_add(), f.reduce_min(), f.reduce_max())
    w = i32x4.from_array([1, 2, 3, 4])
    println(w.shuffle([3, 2, 1, 0]).to_array(), w.shuffle([0, 0, 3, 3]).to_array())
    table: Array[f32] = [10.0, 11.0, 12.0, 13.0]
    idx = i32x4.from_array([3, 0, 1, 3])
    println(f32x4.gather(table, idx).to_array())
    out: Array[f32] = [0.0, 0.0, 0.0, 0.0]
    f32x4.from_array([11.0, 13.0, 0.0, 0.0]).scatter(out, i32x4.from_array([3, 1, 0, 2]))
    println(out)
    a = f64x2.from_array([2.0, 3.0])
    println(a.fma(a, f64x2.from_array([3.0, 8.0])).to_array())
    x = f32x4.from_array([4.0, 16.0, 0.25, 100.0])
    r = x.rsqrt_approx().to_array()
    c = x.rcp_approx().to_array()
    close = true
    for i in 0..4:
        exact_r = 1.0 / math.sqrt(x.to_array()[i] as f64)
        exact_c = 1.0 / (x.to_array()[i] as f64)
        within = fn(value: f64, exact: f64) => value >= exact * 0.999 and value <= exact * 1.001
        close = close and within(r[i] as f64, exact_r) and within(c[i] as f64, exact_c)
    println(close, i64x4.from_array([3, 1, 4, 1]).reduce_max() == 4)
    m = w < i32x4.splat(3)
    println(m.select(w, i32x4.splat(10)).to_array())
