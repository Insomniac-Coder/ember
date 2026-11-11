#$ test: run-pass
#$ rules: ATT-3, SIMD-5
#$ stdout: [2, 4]
# `[ATT-3]` — the attribute attaches to the next compound statement: a `for`.

fn double(xs: Span[int], ref mut out: MutSpan[int]):
    @simd(assert)
    for i in 0..xs.len():
        out[i] = xs[i] * 2

fn main():
    a = [1, 2]
    b = [0, 0]
    double(a, ref mut b)
    println(b)
