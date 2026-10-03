#$ test: run-pass
#$ rules: SIMD-5, SIMD-7, OPT-2
#$ profiles: debug, release, shipping
#$ stdout: -1 67 155
#$ assert-c: !contains(" << ")
#$ assert-c-count: contains("; ++_") == 3
# Reading an element through a shared reference and then rewriting it is
# the same access as the direct indexed read. The group saves its elements
# before running, and the second reference reads the first write's result.

fn main():
    xs: Array[int] = []
    d: Array[int] = []
    for i in 0..40:
        xs.push(i)
        d.push(3 * i)
    for i in 0..40:
        before: ref int = ref xs[i]
        next = before + d[i]
        xs[i] = next
        after: ref int = ref xs[i]
        last = after - 1
        xs[i] = last
    println(xs[0], xs[17], xs[39])
