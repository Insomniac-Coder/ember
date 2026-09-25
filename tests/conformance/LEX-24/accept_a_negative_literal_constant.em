#$ test: run-pass
#$ rules: LEX-24, CT-1, RNG-1
#$ profiles: debug, release, shipping
#$ stdout: -5 -10 -1.5 -3.0 -128 -0.25
#$ 0.5 -20 true
# D-327 — `const N: i64 = -5` and `const F = -1.5` are literal constants,
# as `5` and `1.5` are: a minus on a numeric literal forms one constant
# (`[LEX-24]`). Untyped, it takes its type where it is used (ODR-037), and it
# may bound a range type. They were `E2130`, "must be a literal".

const N: i64 = -5
const F = -1.5
const LEAST: i8 = -128
const QUARTER: f32 = -0.25
const LOW = -20

type Offset = i32 in LOW ..= 20

fn main():
    x: f32 = 0.5
    println(N, N * 2, F, F * 2.0, LEAST, QUARTER)
    println(x + QUARTER - F * 0.5 - 0.5, LOW, Offset.checked(-20).is_ok())
