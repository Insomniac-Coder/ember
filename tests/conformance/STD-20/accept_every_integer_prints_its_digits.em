#$ test: run-pass
#$ rules: STD-20
#$ profiles: debug, release
#$ stdout: -9223372036854775808 9223372036854775807 0 -1 7
#$ stdout: 18446744073709551615 0 4294967295 -2147483648
#$ stdout: -9223372036854775808 18446744073709551615 -10 100
# `[STD-20]` — an integer prints as its decimal digits, a `-` before them
# below zero, the least and the greatest of each type included, whether it
# is printed or formatted into text.

fn main():
    println(i64.MIN, i64.MAX, 0, -1, 7)
    println(u64.MAX, u64.MIN, u32.MAX, i32.MIN)
    a = i64.MIN
    b = u64.MAX
    c = -10
    d: u8 = 100
    println(f"{a} {b} {c} {d}")
