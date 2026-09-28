#$ test: run-fail
#$ rules: RNG-4, TYP-8
#$ profiles: debug, release, shipping
#$ stdout: 3
#$ panics: division by zero
# `[RNG-4]` — `parts >= 0` leaves zero possible: the division keeps its check.

fn share(total: int, parts: int) -> int:
    if parts >= 0:
        return total // parts
    return 0

fn main():
    println(share(10, 3))
    println(share(10, 0))
