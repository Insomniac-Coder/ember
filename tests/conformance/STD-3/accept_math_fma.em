#$ test: run-pass
#$ rules: STD-3, STD-21
#$ stdout: 5.551115123125783e-17 0.0 10.0 1.4901161e-08
# `[STD-3]` — `math.fma(a, b, c)` is `a * b + c` rounded once, on every
# target: `0.1 * 10.0` is not exactly 1, and `fma` keeps the difference that
# the two roundings of `0.1 * 10.0 - 1.0` lose. It takes any number type
# (ODR-038): integers answer in `f64`, an `f32` in `f32`.

import math

fn main():
    x: f32 = 0.1
    println(math.fma(0.1, 10.0, -1.0), 0.1 * 10.0 - 1.0, math.fma(2, 3, 4), math.fma(x, 10.0, -1.0))
