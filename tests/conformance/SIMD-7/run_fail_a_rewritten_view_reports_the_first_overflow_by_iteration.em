#$ test: run-fail
#$ rules: SIMD-7
#$ profiles: debug, release, shipping
#$ panics: integer overflow in `-`
#$ assert-c: !contains(" << ")
#$ assert-c-count: contains("; ++_") == 3
# Two checks on a view read and written: the `-` (second in the body)
# overflows at iteration 5, the `+` (first in the body) at iteration 9, in the
# same group. The group has already changed `xs` when it finds the overflow,
# so it puts back the elements it saved at its start and re-runs checked,
# which reports the `-`, the first overflow by iteration.

fn main():
    xs: Array[int] = []
    d: Array[int] = []
    e: Array[int] = []
    for i in 0..40:
        xs.push(0)
        d.push(0)
        e.push(0)
    xs[5] = -9223372036854775807 - 1
    e[5] = 1
    xs[9] = 9223372036854775807
    d[9] = 1
    for i in 0..40:
        xs[i] = xs[i] + d[i]
        xs[i] = xs[i] - e[i]
    println(xs[0])
