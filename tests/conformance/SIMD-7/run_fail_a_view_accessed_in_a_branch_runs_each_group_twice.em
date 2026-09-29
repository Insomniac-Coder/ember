#$ test: run-fail
#$ rules: SIMD-7
#$ profiles: debug, release, shipping
#$ panics: integer overflow in `-`
#$ assert-c: !contains(" << ")
#$ assert-c-count: contains("4096LL") == 4
# `xs[i]` is read and written only inside the `if`, so not every iteration
# accesses it and the group's elements are not all known to exist: they
# cannot be saved. Each group first runs storing nothing, then for real; the
# `^ 4096` appears in four copies of the body (checked, unchecked, both runs).
# The `-` overflows at iteration 5 and the `+` at iteration 9: the `-` reports.
# `c[3]` is 0 so that the range facts cannot decide the `if` (with every
# element 1 they fold it, and every iteration accesses `xs[i]`).

fn main():
    xs: Array[int] = []
    c: Array[int] = []
    d: Array[int] = []
    e: Array[int] = []
    for i in 0..40:
        xs.push(0)
        c.push(1)
        d.push(0)
        e.push(0)
    c[3] = 0
    xs[5] = -9223372036854775807 - 1
    e[5] = 4097
    xs[9] = 9223372036854775807
    d[9] = 1
    for i in 0..40:
        if c[i] > 0:
            xs[i] = (xs[i] + d[i]) ^ 4096
            xs[i] = xs[i] - e[i]
    println(xs[0])
