#$ test: run-pass
#$ rules: SIMD-7, SIMD-5, OPT-2
#$ profiles: debug, release, shipping
#$ stdout: -1 67 155
#$ assert-c: !contains(" << ")
#$ assert-c-count: contains("for (;") == 3
# `xs[i]` is written and then read again in the same iteration, with two
# checks. Each group runs once, for real, its elements of `xs` saved at its
# start; the second statement reads the first one's result. Three C `for`
# loops: the checked copy, the unchecked copy and the group.

fn main():
    xs: Array[int] = []
    d: Array[int] = []
    for i in 0..40:
        xs.push(i)
        d.push(3 * i)
    for i in 0..40:
        xs[i] = xs[i] + d[i]
        xs[i] = xs[i] - 1
    println(xs[0], xs[17], xs[39])
