#$ test: run-fail
#$ rules: DSJ-5, DSJ-8
#$ profiles: debug, release, shipping
#$ panics: assert_disjoint_or_panic: the views overlap

# `[DSJ-5]`, `[DSJ-8]` — overlapping views panic, in every profile.

from std.mem import assert_disjoint_or_panic

fn main():
    x: Array[int] = [1, 2, 3, 4]
    (p, q) = assert_disjoint_or_panic(x[0..3], x[2..4])
    println(p, q)
