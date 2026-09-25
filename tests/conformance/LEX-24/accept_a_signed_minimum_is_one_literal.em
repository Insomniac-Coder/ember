#$ test: run-pass
#$ rules: LEX-24, TYP-1
#$ profiles: debug, release, shipping
#$ stdout: -128 -32768 -2147483648 -9223372036854775808
#$ -170141183460469231731687303715884105728 -5 0
# D-311 — `[LEX-24]`: a minus directly before an untyped integer literal
# forms one negative constant of the literal's type, so each signed type's
# least value is written as it prints. `128` alone does not fit an `i8`;
# `-128` does.

fn main():
    a: i8 = -128
    b: i16 = -32768
    c: i32 = -2147483648
    d: i64 = -9223372036854775808
    e: i128 = -170141183460469231731687303715884105728
    zero: u8 = -0
    println(a, b, c, d)
    println(e, -5, zero)
