#$ test: compile-fail
#$ rules: IFC-4
# `[IFC-4]` (ODR-049) — an implementation takes a default by following it to
# a type, so a default that leads back to itself names none: `E2043`, once
# for each associated type on the cycle. `C` is not on it.

interface Loop:
    type A = B #$ error[E2043]: the default of `Loop`'s `A` leads back to `A`
    type B = Array[A] #$ error[E2043]: the default of `Loop`'s `B` leads back to `B`
    type C = Self

fn main():
    println(1)
