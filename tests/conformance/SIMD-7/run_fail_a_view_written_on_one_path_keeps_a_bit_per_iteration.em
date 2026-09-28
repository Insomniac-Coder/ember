#$ test: run-fail
#$ rules: SIMD-7
#$ profiles: debug, release, shipping
#$ panics: integer overflow in `-`
#$ assert-c: contains(" << ")
# `xs[i]` is written in the first `if` and read in the second: not every
# iteration accesses it, so its elements cannot be saved, and a run that
# stores nothing would need its value from memory on one path and from the
# first `if` on another. Such a group keeps one overflow bit per check and
# iteration, and reports the first set bit by iteration: the `-` at
# iteration 5, not the `+` (first in the body) at iteration 9.

fn main():
    xs: Array[int] = []
    zs: Array[int] = []
    c: Array[int] = []
    d: Array[int] = []
    e: Array[int] = []
    f: Array[int] = []
    for i in 0..40:
        xs.push(0)
        zs.push(0)
        c.push(0)
        d.push(0)
        e.push(0)
        f.push(0)
    c[3] = 1
    xs[9] = 9223372036854775807
    d[9] = 1
    e[9] = 1
    zs[5] = -9223372036854775807 - 1
    f[5] = 1
    for i in 0..40:
        if c[i] > 0:
            xs[i] = 7
        if e[i] > 0:
            xs[i] = xs[i] + d[i]
        zs[i] = zs[i] - f[i]
    println(xs[0])
