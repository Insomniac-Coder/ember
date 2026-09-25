#$ test: run-pass
#$ rules: CT-7, CT-1, TYP-8, TYP-10
#$ profiles: debug, release, shipping
#$ stdout: 2147483647 -9223372036854775808 2147483648 0 -128 0 -1
# `[CT-7]`'s other side: each of these reaches its type's edge and stays in
# it, so each is a value. `MIN % -1` is 0, not an overflow; a shift by one
# less than the width is in range; `-0` is an unsigned 0.

const EDGE: i32 = 2147483646 + 1
const LEAST: i64 = i64.MIN // 1
const HIGH_BIT: u32 = 1 << 31
const REM: i64 = i64.MIN % -1
const LOW: i8 = -64 * 2
const ZERO: u8 = -(3 - 3)
const TOP: i8 = -1 >> 7

fn main():
    println(EDGE, LEAST, HIGH_BIT, REM, LOW, ZERO, TOP)
