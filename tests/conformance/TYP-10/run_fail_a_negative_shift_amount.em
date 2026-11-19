#$ test: run-fail
#$ rules: TYP-10
#$ profiles: debug, development, release
#$ panics: `shift out of range`: `x << n` on line 10 shifts an `i32` by -1; use `wrapping_shl`
# D-309 — `[TYP-10]`: a negative shift amount panics in every profile, as an
# amount at or past the width does. It was C's undefined shift by a negative
# amount.

fn shift(x: i32, n: i32) -> i32:
    return x << n

fn main():
    println(shift(1, 30))
    println(shift(1, -1))
