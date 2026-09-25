#$ test: run-fail
#$ rules: STD-26
#$ panics: len: the range has more values than an `int` can hold
# `[STD-26]` — `len` of `a..=b` counts `b` too. The count is checked before
# that `+ 1`, so a range one value too long for an `int` panics with `len`'s
# message, not as an overflow of the count (D-272 made 128-bit ranges reach
# this: their count stops at `usize`'s maximum).

fn main():
    lo: i128 = -170141183460469231731687303715884105727 - 1
    hi: i128 = 170141183460469231731687303715884105727
    println(len(0..=9223372036854775806))
    println(len(lo..=hi))
