#$ test: run-fail
#$ rules: TYP-10
#$ profiles: debug, release, shipping
#$ panics: integer overflow in `shift`
# D-309 — `[TYP-10]`: a negative shift amount panics in every profile, as an
# amount at or past the width does. It was C's undefined shift by a negative
# amount.

fn shift(x: int, n: i32) -> int:
    return x << n

fn main():
    println(shift(1, 62))
    println(shift(1, -1))
