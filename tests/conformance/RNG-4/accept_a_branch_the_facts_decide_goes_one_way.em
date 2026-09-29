#$ test: run-pass
#$ rules: RNG-4
#$ profiles: debug, release, shipping
#$ stdout: 207
#$ assert-c: !contains("a path no run takes")
# `[RNG-4]` — a `u8` widened to `int` is below 1000, so the `if` always
# returns: the branch goes one way, and the code only the other way reached
# is gone from the C.

fn widen(n: u8) -> int:
    x: int = n
    if x < 1000:
        return x
    println("a path no run takes")
    return 0

fn main():
    println(widen(7) + widen(200))
