#$ test: run-pass
#$ rules: IFC-4, TYP-17
#$ stdout: 4.5 1.25 4.5 1.25
# `[IFC-4]` (ODR-038) — `T.Real` names the associated type `Real` of a type
# parameter's bound. Each call reads it from the argument's type (`type Real
# = f64` for an `i32`), and inside the body it has what `Real` is declared to
# have (`type Real: Halve`).

interface Halve:
    fn half(self) -> Self

interface Measure:
    type Real: Halve
    fn to_real(self) -> Real

extend f64 implements Halve:
    fn half(self) -> f64:
        return self / 2.0

extend f32 implements Halve:
    fn half(self) -> f32:
        return self / 2.0

extend i32 implements Measure:
    type Real = f64
    fn to_real(self) -> f64:
        return self as f64

extend f32 implements Measure:
    type Real = f32
    fn to_real(self) -> f32:
        return self

fn half_of[T: Measure](x: T) -> T.Real:
    value: T.Real = x.to_real()
    return value.half()

fn main():
    n: i32 = 9
    y: f32 = 2.5
    a: f64 = half_of(n)
    b: f32 = half_of(y)
    println(a, b, half_of(n), half_of(y))
