#$ test: compile-fail
#$ rules: FFI-8, ENM-2
# `[FFI-8]` — a `@non_exhaustive` enum may hold a value no variant names: a
# `match` covering every variant still needs `_` (ODR-137). An exhaustive
# `@repr` enum has no foreign representation: C may hand back any value.
# `@non_exhaustive` is a C enum's: unit-only, with its `@repr`.

@repr(i32)
@non_exhaustive
enum Color:
    Red = 1
    Green = 2

@non_exhaustive  #$ error[E0104]
enum Loose:
    A
    B

@repr(i32)
enum Shape:
    Round = 0
    Square = 1

unsafe extern "C":
    safe fn shape_of(n: i32) -> Shape  #$ error[E5015]

fn name(c: Color) -> str:
    match c:  #$ error[E2090]
        Color.Red:
            return "red"
        Color.Green:
            return "green"

fn main():
    println(name(Color.Red))
