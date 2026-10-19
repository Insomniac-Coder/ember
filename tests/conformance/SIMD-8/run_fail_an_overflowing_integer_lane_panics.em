#$ test: run-fail
#$ rules: SIMD-8, TYP-8
#$ profiles: debug, development, release
#$ stdout: [3, 4, 5, 6]
#$ panics: integer overflow in `+`
# `[SIMD-8]` — integer lanes panic on overflow (`[TYP-8]`), in every profile, as scalar arithmetic
# does: one lane at `i32.MAX` plus one stops the program.
from std.simd import i32x4

fn main():
    a = i32x4.from_array([1, 2, 3, 4])
    println((a + i32x4.splat(2)).to_array())
    b = i32x4.from_array([1, 2_147_483_647, 3, 4])
    c = b + i32x4.splat(1)
    println(c.to_array())
