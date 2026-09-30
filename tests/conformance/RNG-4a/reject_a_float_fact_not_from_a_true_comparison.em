#$ test: compile-fail
#$ rules: RNG-4a, RNG-4, RNG-9, RNG-10
# `[RNG-4a]` — "For floats, a range fact comes only from the true arm of a
# comparison": NaN makes every comparison false, so a false arm proves
# nothing about it, and one comparison leaves the other end open to an
# infinity. D-392: each of these was accepted, and `in_unit(0.0 / 0.0)` gave
# a `Unit` holding NaN, `bounded(inf)` a `Finite` holding infinity.
# Arithmetic, `min`, `max` and `clamp` give a float no fact either; the true
# arm of a two-sided comparison does (`RNG-4/accept_float_branch_refinement`),
# and `T.clamped` or `T.checked` make a range value from any float.

type Unit = f32 in 0.0 ..= 1.0
type Finite = f64 in 0.0 ..= 1.7976931348623157e308
type Ratio = f32 in -10.0 ..= 10.0

fn in_unit(x: f32) -> Unit:
    if x < 0.0:
        return Unit.clamped(0.0)
    else:
        if x > 1.0:
            return Unit.clamped(1.0)
        else:
            return x          #$ error[E2215]: a `Unit` cannot be built from a value this is not known to be in range

fn bounded(x: f64) -> Finite:
    if x >= 0.0:
        return x              #$ error[E2215]: a `Finite` cannot be built from a value this is not known to be in range
    return Finite.clamped(0.0)

fn halved(u: Unit) -> Unit:
    return u * 0.5            #$ error[E2215]: a `Unit` cannot be built from a value this is not known to be in range

fn tenth(r: Ratio) -> Ratio:
    return r / 10.0           #$ error[E2215]: a `Ratio` cannot be built from a value this is not known to be in range

fn clamped_by_hand(x: f32) -> Unit:
    return clamp(x, 0.0, 1.0) #$ error[E2215]: a `Unit` cannot be built from a value this is not known to be in range

fn chained(x: f32) -> Unit:
    return min(max(x, 0.0), 1.0) #$ error[E2215]: a `Unit` cannot be built from a value this is not known to be in range

fn main():
    a: f32 = in_unit(0.5)
    b: f64 = bounded(2.0)
    c: f32 = halved(Unit.clamped(0.5))
    d: f32 = tenth(Ratio.clamped(5.0))
    e: f32 = clamped_by_hand(0.5)
    f: f32 = chained(0.5)
    println(a, b, c, d, e, f)
