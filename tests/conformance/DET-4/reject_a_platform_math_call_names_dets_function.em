#$ test: compile-fail
#$ rules: DET-4, DET-2
#$ error[E4070]: `@deterministic` function `fall_off` reaches a nondeterministic operation
#$ help: `fall_off` is `@deterministic`, and `math.cbrt(d)` (line 11) is the platform's `cbrt`, whose last bits differ between C libraries; write `det.cbrt(d)`
# `[DET-4]` — `E4070`'s help for a `std.math` call is `det`'s function of the same name.

import math

@deterministic
fn fall_off(d: f64) -> f64:
    return math.cbrt(d)

fn main():
    println(fall_off(8.0))
