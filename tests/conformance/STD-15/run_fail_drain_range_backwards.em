#$ test: run-fail
#$ rules: STD-15
#$ panics: drain range starts after it ends
# `[STD-15]` (ODR-031) — a `drain` range that starts after it ends panics
# rather than taking nothing.

fn main():
    xs = [1, 2, 3, 4, 5]
    first = 4
    taken = xs.drain(first..2)
    println(taken)
