#$ test: compile-fail
#$ rules: IFC-5, TYP-22, DIA-2
# `[IFC-5]` — `dyn I` with no owner word is written only as a parameter's type; a bare `dyn I` in a
# field, a result or a payload is `E2173`, whose message gives the two lines to write.

interface Shape:
    fn area() -> int

struct Square:
    side: int

extend Square implements Shape:
    fn area() -> int:
        return this.side * this.side

struct Scene:
    main: dyn Shape    #$ error[E2173]: `main: dyn Shape` does not say whether `Scene` owns or views its shape; write `main: owns dyn Shape` to own it, or `main: ref dyn Shape` for a view

enum Slot:
    Filled(dyn Shape)    #$ error[E2173]
    Empty

fn make() -> dyn Shape:    #$ error[E2173]: `make` returns `dyn Shape`, which does not say whether the caller gets the shape or a view of one; write `-> owns dyn Shape` to hand it over, or `-> ref dyn Shape from` the parameter it points into for a view
    return Square(2)

fn main():
    pass
