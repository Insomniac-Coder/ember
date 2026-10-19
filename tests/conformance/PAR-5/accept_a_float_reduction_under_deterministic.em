#$ test: run-pass
#$ rules: PAR-5, PAR-3
#$ profiles: debug, development
#$ stdout: true
#$ stdout: true
# `[PAR-5]` — a `@parallel` loop's chunks are a function of the trip count and `chunk` only, and
# its reductions combine them in chunk order, so it computes the same bits on every machine and is
# not `Nondet`: a `@deterministic` function may hold one, a float reduction included, and two runs
# agree to the bit.

@deterministic
fn total(xs: Span[f64]) -> f64:
    sum = 0.0
    @parallel(chunk=64, reduce=[sum: +])
    for i in 0..xs.len():
        sum += xs[i] * 0.1
    return sum

fn main():
    values: Array[f64] = []
    for i in range(10000):
        values.push(1.0 / (i + 1) as f64)
    first = total(values.as_span())
    println(first == total(values.as_span()))
    println(first > 0.9)
