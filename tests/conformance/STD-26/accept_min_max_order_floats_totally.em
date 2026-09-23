#$ test: run-pass
#$ rules: STD-26, TYP-37
#$ profiles: debug, release, shipping
#$ stdout: true true false true
# `min` and `max` order floats by IEEE totalOrder (`[TYP-37]`): `-0.0` is less
# than `+0.0`, and `abs` clears the sign of `-0.0`.

fn negative_zero(x: float) -> bool:
    return 1.0 / x < 0.0

fn main():
    println(negative_zero(min(-0.0, 0.0)), negative_zero(min(0.0, -0.0)), negative_zero(max(-0.0, 0.0)), negative_zero(-abs(-0.0)))
