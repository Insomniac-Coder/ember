#$ test: run-pass
#$ rules: RNG-4a, RNG-4, RNG-3, RNG-3a, RNG-6
#$ profiles: debug, release, shipping
#$ stdout: 0.5 0.0 0.0
#$ stdout: true false false
# `[RNG-4a]` — the true arm of a comparison gives a float a fact, and both
# comparisons of `0.0 <= x and x <= 1.0` exclude NaN, so `x` is a `Unit` in
# that arm with no check. Any other float becomes one through `clamped`,
# which sends NaN to the lower end (`[RNG-3a]`), or `checked`, where NaN is in
# no range (`[RNG-6]`).

type Unit = f32 in 0.0 ..= 1.0

fn in_unit(x: f32) -> Unit:
    if 0.0 <= x and x <= 1.0:
        return x
    return Unit.clamped(x)

fn accepts(x: f32) -> bool:
    match Unit.checked(x):
        Ok(_):
            return true
        Err(_):
            return false

fn main():
    zero: f32 = 0.0
    nan = zero / zero
    a: f32 = in_unit(0.5)
    b: f32 = in_unit(nan)
    c: f32 = in_unit(-3.0)
    println(a, b, c)
    println(accepts(0.5), accepts(nan), accepts(2.0))
