#$ test: compile-fail
#$ rules: STD-4
# `[STD-4]` — there is a `NonZero` for each integer type and for no other
# type: a float has two zeros, `0.0` and `-0.0`, and no niche to give.

from std.core import NonZero

fn main():
    _half = NonZero.new(0.5) #$ error[E2040]: `f64` does not implement `std.core.Integer`, which `NonZero`'s `T` requires
