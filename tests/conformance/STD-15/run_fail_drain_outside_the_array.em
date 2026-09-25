#$ test: run-fail
#$ rules: STD-15
#$ panics: drain range reaches outside the array
#$ stdout: before
# `[STD-15]` (ODR-031) — a `drain` range reaching past the end panics, and
# nothing is taken first.

fn main():
    xs = [1, 2, 3, 4, 5]
    println("before")
    taken = xs.drain(3..=5)
    println(taken)
