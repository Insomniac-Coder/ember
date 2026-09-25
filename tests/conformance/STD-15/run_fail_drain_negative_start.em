#$ test: run-fail
#$ rules: STD-15, TYP-31
#$ panics: drain range reaches outside the array
# `[STD-15]` (ODR-031) — a negative start is outside the array, as a negative
# index is (`[TYP-31]`).

fn main():
    xs = [1, 2, 3]
    start = -1
    taken = xs.drain(start..2)
    println(taken)
