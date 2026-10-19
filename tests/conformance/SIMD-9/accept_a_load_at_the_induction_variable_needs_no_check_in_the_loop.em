#$ test: run-pass
#$ rules: SIMD-9, OPT-2
#$ profiles: development
#$ stdout: 2256.0
#$ assert-c-count: contains("_load(") == 2
#$ assert-c-count: contains("ember_panic_bounds(") == 2
# `[SIMD-9]` — "A `load` or `store` at a loop's induction variable is covered by the entry test of
# `[OPT-2]`": the loop is versioned on one test before it that every slice's end is within its
# view (for a stepped loop, the last value's), so the copy that runs then checks nothing per turn;
# the two slice-end checks are left only in the checked copy, which runs when the test fails.
from std.simd import f32x8

fn scale(xs: Span[f32], mut out: MutSpan[f32], k: f32):
    kv = f32x8.splat(k)
    for i in (0..xs.len() - 7).step_by(8):
        (f32x8.load(xs[i..i + 8]) * kv).store(out[i..i + 8])

fn main():
    xs: Array[f32] = []
    out: Array[f32] = []
    for i in 0..48:
        xs.push(i as f32)
        out.push(0.0)
    scale(xs, out, 2.0)
    total = 0.0
    for x in out:
        total += x as f64
    println(total)
