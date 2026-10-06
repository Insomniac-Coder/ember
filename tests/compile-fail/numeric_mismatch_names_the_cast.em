#$ test: compile-fail
#$ rules: TYP-4
#$ error[E2020]: `+` cannot be applied to `i64` and `u64`
#$ note: `b as i64` makes them agree
#$ error[E2020]: `+` cannot be applied to `i64` and `f64`
#$ note: `a as f64` makes them agree
# `[TYP-4]` — of two whole numbers of different types, one converts to the other's type only where
# every value it can have fits (the owner's ruling of 2026-10-06): `a`, an `int` parameter, may be
# negative and `b`, a `u64` one, may pass `int`'s top, so neither fits the other. A float never
# converts. The note names the exact cast on the narrower operand (an integer becomes a float; the
# right one when both have as many bits).

fn sums(a: int, b: u64, f: f64):
    c = a + b
    g = a + f
    println(c, g)

fn main():
    sums(1, 2, 1.5)
