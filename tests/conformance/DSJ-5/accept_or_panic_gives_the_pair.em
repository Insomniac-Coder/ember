#$ test: run-pass
#$ rules: DSJ-5
#$ profiles: debug, release, shipping
#$ stdout: 4

# `[DSJ-5]` — `assert_disjoint_or_panic` gives the two views back as a pair.

from std.mem import assert_disjoint_or_panic

fn main():
    x: Array[int] = [1, 2, 3]
    y: Array[int] = [4]
    (p, q) = assert_disjoint_or_panic(x[..], y[..])
    println(p.len() + q.len())
