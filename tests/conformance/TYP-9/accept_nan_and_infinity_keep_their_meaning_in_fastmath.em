#$ test: run-pass
#$ rules: TYP-9, TYP-6, RNG-3, RNG-3a, RNG-6, PHIL-10
#$ profiles: debug, release, shipping
#$ stdout: (true, false, false) (false, false, true) (false, true, false)
#$ stdout: 0.0 1.0 0.0 false true false
#$ stdout: 0 9223372036854775807 -9223372036854775808 2
# `@fastmath` relaxes the function's own arithmetic, never what the language
# defines exactly for NaN and infinity (ODR-090): `is_nan`, `is_finite` and
# `is_infinite` answer by the bits; a range value never holds NaN, which
# `clamped` sends to the lower end and `checked` refuses (`[RNG-3a]`,
# `[RNG-6]`, `[PHIL-10]`); and `x as int` is 0 for NaN and saturates for an
# infinity (`[TYP-6]`). The relaxed C file compiles these through the
# runtime or by the bits, and clang and gcc never assume that no value is
# NaN, which would make a NaN argument undefined behaviour.

type Unit = f64 in 0.0 ..= 1.0
type Half = f32 in 0.0 .. 1.0

@fastmath
fn probe(x: f64) -> (bool, bool, bool):
    return (x.is_nan(), x.is_finite(), x.is_infinite())

@fastmath
fn clampit(x: f64) -> Unit:
    return Unit.clamped(x)

@fastmath
fn clamp32(x: f32) -> Half:
    return Half.clamped(x)

@fastmath
fn check(x: f64) -> bool:
    match Unit.checked(x):
        Ok(_):
            return true
        Err(_):
            return false

@fastmath
fn to_int(x: f64) -> int:
    return x as int

fn main():
    zero = 0.0
    nan = zero / zero
    inf = 1.0 / zero
    u: f64 = clampit(nan)
    w: f64 = clampit(inf)
    h: f32 = clamp32(nan as f32)
    println(probe(nan), probe(inf), probe(2.0))
    println(u, w, h, check(nan), check(0.5), check(inf))
    println(to_int(nan), to_int(inf), to_int(-inf), to_int(2.9))
