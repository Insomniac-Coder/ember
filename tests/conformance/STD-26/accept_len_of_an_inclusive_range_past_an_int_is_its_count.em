#$ test: run-pass
#$ rules: STD-26, STD-19
#$ stdout: 9223372036854775807
#$ stdout: 340282366920938463463374607431768211456
# `[STD-26]`, G8-4 (the owner's design, 0.9.10) — `len` of `a..=b` counts `b` too: the whole
# `i128` range has 2^128 values, a `u256`. Where the numbers are written in place, the count is
# in the smallest type that holds it: `len(0..=9223372036854775806)` is an `int`. The second
# panicked before, "len: the range has more values than an `int` can hold".

fn main():
    lo: i128 = -170141183460469231731687303715884105727 - 1
    hi: i128 = 170141183460469231731687303715884105727
    n: int = len(0..=9223372036854775806)
    println(n)
    println(len(lo..=hi))
