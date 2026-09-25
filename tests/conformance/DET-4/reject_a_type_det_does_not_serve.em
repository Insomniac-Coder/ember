#$ test: compile-fail
#$ rules: DET-4
# `[DET-4]` — `std.math.det`'s functions are for `f32` and `f64`; an integer
# is converted first. An `f16` is too, to `f32`.

import math.det

fn main():
    println(det.sin(1))    #$ error[E2040]: `i64` does not implement `std.math.det.Deterministic`, which `T` requires
    n: i32 = 4
    println(det.sqrt(n as f64))
    h: f16 = 0.5
    println(det.exp(h))    #$ error[E2040]: `f16` does not implement `std.math.det.Deterministic`, which `T` requires
