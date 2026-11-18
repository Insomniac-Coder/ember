#$ test: compile-fail
#$ rules: TYP-6
## `[TYP-6]` - nothing converts to `bool`, an integer never converts to an enum, a `char` converts to a `u32`
## only, and `as` between any other pair of types is `E2020`, naming both.

enum Color:
    Red
    Green

enum Big:
    Small
    Huge = 300

fn main():
    n = 7
    a = n as bool                          #$ error[E2020]: `i64` cannot be cast to `bool` with `as`
    c = 'x'
    b = c as int                           #$ error[E2020]: `char` cannot be cast to `i64` with `as`
    d = 7 as Color                         #$ error[E2020]: an integer may be no `Color`; write `Color.from_repr(7)`, an `Option[Color]`, and open it: `if Color.from_repr(7) exists as c:`
    e = Big.Huge as u8                     #$ error[E2020]: `Big` has the discriminant `300`, which `u8` cannot hold: `as` converts a unit-only enum to an integer type that holds every one of them
    f = "5" as int                         #$ error[E2020]: `StrView` cannot be cast to `i64` with `as`
    g = 1.5 as char                        #$ error[E2020]: `f64` cannot be cast to `char` with `as`
    println(a, b, d, e, f, g)
