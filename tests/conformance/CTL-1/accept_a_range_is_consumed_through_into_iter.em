#$ test: run-pass
#$ rules: CTL-1, STD-19
#$ stdout: [1, 2, 3]
#$ stdout: [1, 2, 3]
#$ stdout: 1
# D-444 — a range is iterable, and as a value it is consumed through
# `IntoIterator`: `into_iter()`, and an iterable an adapter is given.

fn main():
    a: Array[int] = (1..4).into_iter().collect()
    println(a)
    b: Array[int] = (1..=3).into_iter().collect()
    println(b)
    println((5..).into_iter().take(1).count())
