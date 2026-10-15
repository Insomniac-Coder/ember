#$ test: run-fail
#$ rules: STD-26
#$ panics: range() step must not be zero
# `[STD-26]` — `step == 0` panics, as a value as in a `for` header.

fn step(n: int) -> int:
    return n - n

fn main():
    r = range(0, 10, step(3))
    for v in r:
        println(v)
