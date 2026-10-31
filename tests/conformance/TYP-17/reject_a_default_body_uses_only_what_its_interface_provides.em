#$ test: compile-fail
#$ rules: TYP-17, IFC-4
# `[TYP-17]` — an interface's default body is generic code over `Self`: it
# may use what the interface (and its parents) provide, nothing else. D-344:
# default bodies were checked only as each implementation's copy, so one
# calling a method the implementing type happened to have compiled, and one
# no type implemented was never checked at all.

interface Named:
    fn name() -> StrView

    fn shout() -> int:
        return this.secret() #$ error[E2040]: `Self` has no method `secret`; its bounds do not provide one

struct Dog:
    n: int

extend Dog:
    fn secret() -> int:
        return this.n

extend Dog implements Named:
    fn name() -> StrView:
        return "dog"

fn main():
    println(Dog(3).shout())
