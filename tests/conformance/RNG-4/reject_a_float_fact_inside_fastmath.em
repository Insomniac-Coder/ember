#$ test: compile-fail
#$ rules: RNG-4, RNG-4a, RNG-10, TYP-9
#$ profiles: debug
#$ error[E2215]: a `Unit` cannot be built from a value this is not known to be in range
#$ error[E2215]: a `Unit` cannot be built from a value this is not known to be in range
# ODR-090 — inside `@fastmath` no fact about a float is derived: the function
# may be compiled as if no value were NaN, so a comparison's arm proves
# nothing about one, and `value` needs a check to become a `Unit`, as does a
# `clamp` of it. Its integer
# facts are derived as anywhere else, since no float mode changes integer
# arithmetic; `@fp(contract)` keeps a comparison's fact, as contraction
# changes no comparison.

type Unit = f32 in 0.0 ..= 1.0
type Small = int in 0 ..= 10

@fastmath
fn as_unit(value: f32) -> Unit:
    if value > 0.0 and value < 1.0:
        return value
    return Unit.clamped(0.0)

@fastmath
fn clamped_by_hand(value: f32) -> Unit:
    return clamp(value, 0.0, 1.0)

@fastmath
fn as_small(n: int) -> Small:
    if n >= 0 and n <= 5:
        return n
    return 0

@fp(contract)
fn as_unit_contract(value: f32) -> Unit:
    if value > 0.0 and value < 1.0:
        return value
    return Unit.clamped(0.0)

fn main():
    a: f32 = as_unit(0.5)
    b: int = as_small(3)
    c: f32 = as_unit_contract(0.5)
    d: f32 = clamped_by_hand(0.5)
    println(a, b, c, d)
