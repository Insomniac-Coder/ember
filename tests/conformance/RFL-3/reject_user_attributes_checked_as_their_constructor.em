#$ test: compile-fail
#$ rules: RFL-3, PHIL-12
## `[RFL-3]` — a user attribute is checked exactly as its constructor call, is written only on a
## `@reflect` type, its fields and variants, holds only numbers, text, `bool`s and `char`s, and an
## attribute that names no built-in attribute and no visible `@attribute` struct is `E0104`.

@attribute
struct Range:
    min: float
    max: float

@attribute
struct Bad:
    items: Array[int]  #$ error[E0104]: field `items` of an `@attribute` struct

@reflect
struct Car:
    @Range(min=0.0, max="high")  #$ error[E2020]
    speed: float
    @Unknown(1)  #$ error[E0104]: `@Unknown` is not an attribute
    wheels: int

struct Plain:
    @Range(min=0.0, max=1.0)  #$ error[E0104]: `@Range` is written on a `@reflect` type
    x: float

fn main():
    pass
