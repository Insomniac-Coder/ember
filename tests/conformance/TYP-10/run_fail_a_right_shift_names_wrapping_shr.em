#$ test: run-fail
#$ rules: TYP-10
#$ profiles: debug, development, release
#$ stdout: 25
#$ panics: `shift out of range`: `a >> n` on line 10 shifts a `u8` by 9; use `wrapping_shr`
# `[TYP-10]` — `>>` by an amount not less than the width panics too, naming `wrapping_shr`; the amount is
# the unsigned `u32` it was written as.

fn shr(a: u8, n: u32) -> u8:
    return a >> n

fn main():
    println(shr(200, 3))
    println(shr(200, 9))
