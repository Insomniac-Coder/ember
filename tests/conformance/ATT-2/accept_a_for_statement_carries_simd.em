#$ test: run-pass
#$ rules: ATT-2, SIMD-5
#$ stdout: [3, 6, 9, 12]
# `[ATT-2]` — a statement may carry `@simd` (and `@parallel`, `@unroll`, `@allow`), the first three
# only on a `for` statement.

fn scale(xs: Span[int], ref mut out: MutSpan[int], k: int):
    @simd(assert)
    for i in 0..xs.len():
        out[i] = xs[i] * k

fn main():
    a = [1, 2, 3, 4]
    b = [0, 0, 0, 0]
    scale(a, ref mut b, 3)
    println(b)
