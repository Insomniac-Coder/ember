#$ test: compile-fail
#$ rules: DIA-24, DIA-12
#$ error[E1010]: `Color` has no variant `Rd`
#$ help: the nearest name is `Red`
#$ error[E1010]: `Color` has no variant `Gren`
#$ help: the nearest name is `Green`
#$ error[E1010]: `Color` has no variant `Purple`
#$ not-help: `Blue`
#$ error[E1010]: cannot find `Colr` in this scope
#$ help: the nearest name is `Color`
# `[DIA-24]` — a variant name that is not one is offered the variants of its enum a close spelling
# apart (`Color.Rd`), by the same distance and order as any name: `Blue` is never offered for
# `Purple`. Before a `.` a type is offered too, a type being a value only there (`Colr.Red`).

enum Color:
    Red
    Green
    Blue

fn main():
    a = Color.Rd
    b = Color.Gren
    c = Color.Purple
    d = Colr.Red
