#$ test: run-fail
#$ rules: RNG-4, TYP-8, TYP-28
#$ profiles: debug, release, shipping
#$ stdout: 3
#$ panics: integer overflow in `//`
# `[RNG-4]` — a negative divisor can be `-1` and the dividend `int.MIN`: the
# floor division keeps its overflow check.

fn half(x: int, d: int) -> int:
    if d < 0:
        return x // d
    return 0

fn main():
    println(half(-7, -2))
    println(half(-9223372036854775807 - 1, -1))
