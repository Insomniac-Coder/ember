#$ test: run-pass
#$ rules: TYP-10, CG-C-1
#$ profiles: debug, development, release
#$ assert-c: contains("((int64_t)((uint64_t)(a) << ")
#$ stdout: -12
#$ -12 -128 -128
#$ -9223372036854775808
# `[TYP-10]` — bits shifted out are discarded and are not an overflow, a negative value's too.
# D-972: C's `<<` of a negative signed value is undefined (C11 6.5.7), so the C backend shifts a
# signed value in the unsigned type of its width and converts back.

fn shifted(a: int, n: int) -> int:
    return a << n

fn small(a: i8, n: int) -> i8:
    return a << n

fn main():
    println(shifted(-3, 2))
    println(small(-3, 2), small(64, 1), small(-128, 0))
    println(shifted(-1, 63))
