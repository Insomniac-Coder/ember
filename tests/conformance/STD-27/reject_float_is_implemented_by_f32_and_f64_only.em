#$ test: compile-fail
#$ rules: STD-27, TYP-17
# `[STD-27]` (ODR-037) — `Float` is `f32`'s and `f64`'s: a literal must be
# able to become the type, which no program type can promise, so
# implementing it is `E2042`. An integer does not meet the bound.

from std.math import Float

struct Money:
    cents: int

extend Money implements Float:    #$ error[E2042]: `Money` cannot implement `Float`
    fn sqrt(self) -> Money:
        return self

fn twice[T: Float](x: T) -> T:
    return x * 2

fn main():
    n: i64 = 3
    println(twice(n))    #$ error[E2040]: `i64` does not implement `std.math.Float`, which `T` requires
