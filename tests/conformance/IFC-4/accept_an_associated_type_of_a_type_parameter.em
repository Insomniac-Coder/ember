#$ test: run-pass
#$ rules: IFC-4, TYP-17
#$ stdout: 4.5 1.25 4.5 1.25
# `[IFC-4]` (ODR-038) — `T.Real` names the associated type `Real` of a type
# parameter's bound. Each call reads it from the argument's type (`type Real
# = f64` for an `i32`), and inside the body it has what `Real` is declared to
# have (`type Real: Halve`).

interface Halve:
    fn half() -> Self

interface Measure:
    type Real: needs Halve
    fn to_real() -> Real

extend f64 implements Halve:
    fn half() -> f64:
        return this / 2.0

extend f32 implements Halve:
    fn half() -> f32:
        return this / 2.0

extend i32 implements Measure:
    type Real = f64
    fn to_real() -> f64:
        return this as f64

extend f32 implements Measure:
    type Real = f32
    fn to_real() -> f32:
        return this

[T: needs Measure]
fn half_of(x: T) -> T.Real:
    value: T.Real = x.to_real()
    return value.half()

fn main():
    n: i32 = 9
    y: f32 = 2.5
    a: f64 = half_of(n)
    b: f32 = half_of(y)
    println(a, b, half_of(n), half_of(y))
