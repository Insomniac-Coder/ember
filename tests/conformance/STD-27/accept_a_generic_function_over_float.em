#$ test: run-pass
#$ rules: STD-27, STD-21, TYP-17
#$ profiles: debug, release, shipping
#$ stdout: 5.0 5.0
#$ 2.0 0.5 1.0
#$ 8.0 0.25
#$ 2.0 2.0
# `[STD-27]` (ODR-037) — a body bounded by `T: Float` has the arithmetic
# operators on `T`, literals that become `T` (`0.5`, `2`), `**`, the
# prelude's `min`/`max`/`clamp`, and the float methods; the same function
# serves `f32` and `f64`. A literal passed for a `Float` parameter is a
# float: `sqrt(4)` is `2.0`.

from std.math import Float, sqrt

fn norm[T: Float](x: T, y: T) -> T:
    return (x * x + y * y).sqrt()

fn halve_and_clamp[T: Float](x: T) -> T:
    return clamp(x * 0.5, 0.0, 1.0)

fn cube_of_half[T: Float](x: T) -> T:
    half = x / 2
    return half ** 3

fn main():
    a: f32 = 3.0
    println(norm(3.0, 4.0), norm(a, 4.0f32))
    println(sqrt(4), halve_and_clamp(1.0), halve_and_clamp(9.0f32))
    println(cube_of_half(4.0), -max(-0.25, -1.0))
    println(sqrt(4.0f32), min(2.0, -(-3.0)))
