#$ test: run-pass
#$ rules: STD-21, TYP-23
#$ stdout: 3.141592653589793 3.1415927 6.283185307179586 2.718281828459045
#$ 0.5 10 0.5 10
#$ 1.0 0.7853981633974483
# `[STD-21]`, V.7 (ODR-037) — `PI`, `TAU` and `E` are untyped constants:
# `x: f32 = PI` is `PI` rounded to `f32`, and a use nothing types is `f64`.
# A program's own `const` written without a type and with a literal value is
# untyped the same way. `math.PI` is reached through the module too.

import math
from math import PI

const HALF = 0.5
const TEN = 10

fn main():
    turn: f32 = PI
    println(math.PI, turn, math.TAU, math.E)
    h: f32 = HALF
    t: u8 = TEN
    println(h, t, HALF, TEN)
    println(math.sin(PI / 2.0), math.atan2(1.0, 1.0))
