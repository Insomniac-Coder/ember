#$ test: run-fail
#$ rules: SIMD-9
#$ profiles: debug, release, shipping
#$ panics: lane 3: index 3 is out of bounds for a length of 3
# `[SIMD-9]` — `v.store(s)` requires `s.len() >= V.LANES`: with three elements the fourth lane has
# nowhere to go, and the store panics naming it before writing anything.
from std.simd import f64x4

fn fill(mut out: MutSpan[f64]):
    f64x4.splat(1.0).store(out[1..])

fn main():
    out: Array[f64] = [0.0, 0.0, 0.0, 0.0]
    fill(out)
    println(out)
