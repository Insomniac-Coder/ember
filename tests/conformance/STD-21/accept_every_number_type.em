#$ test: run-pass
#$ rules: STD-21, IFC-4, STD-27
#$ profiles: debug, release, shipping
#$ stdout: 3.0 4.0 1.4142135623730951 1.4142135 1.4142135623730951 2.0
#$ 1024.0 5.0 0.0 10.0 0.5 0.5
#$ 1.4142135 3.1415927 nan
# ODR-038 — each `std.math` function takes any number type. An integer
# answers in `f64` (`sqrt` of the `i32` 9 is `3.0`), an `f32` in `f32` and an
# `f64` in `f64`: the answer's type is `T.Real`, which each number type's
# `extend … implements Number` block in `std.math` states. A literal is an
# `int` and so answers in `f64`. The square root of a negative number is
# `nan`, as C's is.

import math

fn main():
    n: i32 = 9
    b: u8 = 16
    big: i64 = 2
    y: f32 = 2.0
    x = 2.0
    println(math.sqrt(n), math.sqrt(b), math.sqrt(big), math.sqrt(y), math.sqrt(x), math.sqrt(4))
    w: u16 = 1
    println(math.pow(2, 10), math.hypot(3, 4), math.sin(0), math.lerp(0, 10, 1), math.smoothstep(0.0, 1.0, 0.5), math.rsqrt(4) * w.to_real())
    small: f32 = math.sqrt(y)
    turn: f32 = math.PI
    println(small, turn, math.sqrt(-3.0))
