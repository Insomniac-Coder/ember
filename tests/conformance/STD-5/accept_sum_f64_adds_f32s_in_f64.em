#$ test: run-pass
#$ rules: STD-5
#$ profiles: debug, release
#$ stdout: 16777216.0 16777218.0
#$ stdout: 0.0
# `[STD-5]` — `sum` adds in the element type, and `sum_f64()` adds an
# iterator's `f32`s in an `f64`: past 2^24 an `f32` total drops each `1.0`,
# where the `f64` one keeps them (D-471).

fn main():
    ys: Array[f32] = [16777216.0, 1.0, 1.0]
    println(ys.iter().copied().sum(), ys.iter().copied().sum_f64())
    none: Array[f32] = []
    println(none.iter().copied().sum_f64())
