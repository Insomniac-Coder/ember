#$ test: compile-fail
#$ rules: TYP-22
# `[TYP-22]` — an interface that requires `Ord` (`interface Shape: needs (Eq, Ord):`) is usable as
# `dyn`: a method of a required interface is reached through `dyn I` only where it names `Self`
# nowhere but as `this`. `cmp` takes `other: Self`, so calling it through a `dyn Shape` value is
# `E2050` at the call, while forming `dyn Shape` is fine.

interface Shape: needs (Eq, Ord):
    fn area() -> int

fn first(a: ref dyn Shape, b: ref dyn Shape) -> int:
    return a.area()

fn order(a: ref dyn Shape, b: ref dyn Shape):
    a.cmp(b)    #$ error[E2050]: `cmp` takes `other: Self`, and through a `dyn Shape` the kind of `other` is unknown; test the kinds first

fn main():
    pass
