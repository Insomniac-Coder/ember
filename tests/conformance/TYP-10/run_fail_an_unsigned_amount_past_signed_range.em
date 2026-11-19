#$ test: run-fail
#$ rules: TYP-10
#$ profiles: debug, development, release
#$ panics: `shift out of range`: `a << n` on line 8 shifts an `i64` by 18446744073709551615; use `wrapping_shl`
# `[TYP-10]` — the amount is shown in its own type: a `u64` of 2^64 - 1 is not -1.

fn shl(a: i64, n: u64) -> i64:
    return a << n

fn main():
    println(shl(1, 18446744073709551615))
