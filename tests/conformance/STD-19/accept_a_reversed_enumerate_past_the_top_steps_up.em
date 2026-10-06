#$ test: run-pass
#$ rules: STD-19, CTL-3b, TYP-8
#$ profiles: debug, release, shipping
#$ stdout:
#$ 9223372036854775808 3
#$ 9223372036854775807 2
#$ 9223372036854775806 1
#$ [(9223372036854775808, 3), (9223372036854775807, 2), (9223372036854775806, 1)]
# ODR-091, `[STD-19]` (the owner's ruling of 2026-10-06, replacing G8-4 decision F) — run
# backwards, `enumerate` gives its greatest number first (`number + len - 1`). Numbers that could
# pass `int`'s top are kept in a bigger kind of number, `u64` here, so each item gets its real
# number and nothing overflows: in a `for` (a counted loop) and through the library (`to_array`).

fn main():
    xs: Array[int] = [1, 2, 3]
    for i, x in xs.iter().enumerate(9223372036854775806).rev():
        println(i, x)
    println(xs.iter().copied().enumerate(9223372036854775806).rev().to_array())
