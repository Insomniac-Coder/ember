#$ test: run-pass
#$ rules: MONO-9, MONO-7
#$ profiles: release
#$ assert-c-count: contains("(ember_dyn x) {") == 1
#$ stdout: 4.0 9.0 4.0
# `[MONO-9]` — a shared body is its own symbol and is deduplicated like any instance: three calls
# through two types, one body.

interface Shape:
    fn area() -> f64

struct Square:
    side: f64

extend Square implements Shape:
    fn area() -> f64:
        return this.side * this.side

@never_specialize
[T: needs Shape]
fn area_of(x: T) -> f64:
    return x.area()

fn main():
    println(area_of(Square(2.0)), area_of(Square(3.0)), area_of(Square(2.0)))
