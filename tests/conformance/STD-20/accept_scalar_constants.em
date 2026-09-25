#$ test: run-pass
#$ rules: STD-20
#$ stdout: -128 127 65535 9223372036854775807 -9223372036854775808 0
#$ -170141183460469231731687303715884105728 340282366920938463463374607431768211455
#$ 1.7976931348623157e+308 -1.7976931348623157e+308 3.4028235e+38 -3.4028235e+38
#$ 2.220446049250313e-16 1.1920929e-07 inf nan inf true true
# ODR-039, `[STD-20]` — `MIN` and `MAX` of every integer type, through its
# name or an alias (`int.MAX`), and a float's `INF`, `NAN`, `EPSILON` (the
# gap between 1.0 and the next value), `MAX`, and `MIN`, the least finite
# value (`-MAX`), as an integer's `MIN` is its least.

fn main():
    println(i8.MIN, i8.MAX, u16.MAX, int.MAX, int.MIN, u8.MIN)
    println(i128.MIN, u128.MAX)
    println(f64.MAX, f64.MIN, f32.MAX, f32.MIN)
    println(f64.EPSILON, f32.EPSILON, f64.INF, f32.NAN, float.INF, 1.0 + f64.EPSILON != 1.0, 1.0 + f64.EPSILON / 2.0 == 1.0)
