#$ test: compile-fail
#$ rules: IFC-4, TYP-17
# `[IFC-4]` (ODR-038) — a type implementing an interface with an associated
# type says what it is, and its value meets the declared bound; a type
# parameter has only the associated types its bounds declare.

interface Halve:
    fn half(self) -> Self

interface Measure:
    type Real: Halve
    fn to_real(self) -> Real

extend f64 implements Halve:
    fn half(self) -> f64:
        return self / 2.0

extend i32 implements Measure:    #$ error[E2040]: `i32` implements `Measure` but does not say what `Real` is
    fn to_real(self) -> f64:
        return self as f64

extend u8 implements Measure:    #$ error[E2040]: `u8`'s `Real` is `bool`, which does not implement `Halve`
    type Real = bool
    fn to_real(self) -> bool:
        return self > 0

fn wrong[T: Measure](x: T) -> T.Size:    #$ error[E2040]: `T` has no associated type `Size`; its bounds declare none
    return x

fn main():
    println(1)
