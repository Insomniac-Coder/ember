#$ test: run-pass
#$ rules: TYP-6
#$ profiles: debug, release, shipping
#$ stdout: 2147483647
#$ -2147483648
#$ 0
#$ 255
#$ 0
#$ -7
#$ 0
# `-3.5 as u8` is `(-3.5) as u8`: a prefix `-` binds tighter than `as` (Part
# III's table).
#
# A float converts to an integer by rounding toward zero and saturating at the
# target's bounds; NaN becomes 0 (D-188: this was C's undefined conversion).

fn main():
    big = 1.0e20
    println(big as i32)
    println((-big) as i32)
    nan = 0.0 / 0.0
    println(nan as i32)
    println(300.5 as u8)
    println((-3.5) as u8)
    println((-7.9) as i64)
    println(-3.5 as u8)
