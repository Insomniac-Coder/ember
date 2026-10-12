#$ test: parse-fail
#$ rules: GRM-33, PHIL-12
## `[GRM-33]` — a function with no body is legal only in an `interface`, an
## `extern` block, or as a `virtual` method of an `abstract class`; anywhere
## else it was accepted and dropped in silence (D-576).

fn missing(x: int) -> int    #$ error[E0110]: `missing` is declared without a body

struct Point:
    x: int

    fn get(self) -> int    #$ error[E0110]: `get` is declared without a body

open class Shape:
    fn area(self) -> f64    #$ error[E0110]: `area` is declared without a body

abstract class Solid:
    fn volume(self) -> f64    #$ error[E0110]: `volume` is declared without a body

fn main():
    pass
