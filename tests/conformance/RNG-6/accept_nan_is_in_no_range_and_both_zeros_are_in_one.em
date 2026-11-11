#$ test: run-pass
#$ rules: RNG-6
#$ profiles: debug, development, release
#$ stdout: false true false true true
## `[RNG-6]` — NaN is in no range. A range from `-0.0` to `+0.0` contains both zeros.

type Unit = f64 in 0.0 ..= 1.0
type Zeros = f64 in -0.0 ..= 0.0

fn unit_has(x: f64) -> bool:
    match Unit.checked(x):
        Ok(_):
            return true
        Err(_):
            return false

fn zeros_have(x: f64) -> bool:
    match Zeros.checked(x):
        Ok(_):
            return true
        Err(_):
            return false

fn main():
    zero: f64 = 0.0
    nan = zero / zero
    negative = -zero
    println(unit_has(nan), unit_has(0.5), zeros_have(nan), zeros_have(zero), zeros_have(negative))
