#$ test: run-fail
#$ rules: RNG-4, TYP-8
#$ profiles: debug, release, shipping
#$ stdout: 6
#$ panics: integer overflow in `+`
# `[RNG-4]` — `x > 0` bounds `x` below only: `x + 1` can still overflow and
# keeps its check.

fn next(x: int) -> int:
    if x > 0:
        return x + 1
    return 0

fn main():
    println(next(5))
    println(next(9223372036854775807))
