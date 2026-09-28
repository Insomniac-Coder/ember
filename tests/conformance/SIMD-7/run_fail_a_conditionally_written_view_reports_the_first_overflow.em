#$ test: run-fail
#$ rules: SIMD-7
#$ profiles: debug, release, shipping
#$ panics: integer overflow in `-`
#$ assert-c: !contains(" << ")
# `xs[i]` is written in the `if` and read after it, so every iteration
# accesses it: each group saves its elements of `xs` and runs once. The
# checked re-run after the overflow reports the first by iteration: the `-`
# at iteration 5, not the `+` at iteration 9.

fn main():
    xs: Array[int] = []
    c: Array[int] = []
    d: Array[int] = []
    e: Array[int] = []
    for i in 0..40:
        xs.push(0)
        c.push(0)
        d.push(0)
        e.push(0)
    c[3] = 1
    xs[5] = -9223372036854775807 - 1
    e[5] = 1
    xs[9] = 9223372036854775807
    d[9] = 1
    for i in 0..40:
        if c[i] > 0:
            xs[i] = 7
        xs[i] = xs[i] + d[i]
        xs[i] = xs[i] - e[i]
    println(xs[0])
