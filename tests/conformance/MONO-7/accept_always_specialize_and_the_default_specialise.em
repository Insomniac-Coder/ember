#$ test: run-pass
#$ rules: MONO-6, MONO-7
#$ assert-c: !contains("ember_dyn _1")
#$ stdout: 4.0 3.0 1.0 3.0
## `[MONO-6]` — with no ceiling set, nothing is shared: each instance of `area_of` is its own body.
## `[MONO-7]` — `@always_specialize` forbids sharing; neither attribute changes what the program
## computes.

interface Shape:
    fn area() -> f64

struct Square:
    side: f64

extend Square implements Shape:
    fn area() -> f64:
        return this.side * this.side

struct Disc:
    r: f64

extend Disc implements Shape:
    fn area() -> f64:
        return 3.0 * this.r * this.r

[T: needs Shape]
fn area_of(x: T) -> f64:
    return x.area()

@always_specialize
[T: needs Shape]
fn exact(x: T) -> f64:
    return x.area()

fn main():
    println(area_of(Square(2.0)), area_of(Disc(1.0)), exact(Square(1.0)), exact(Disc(1.0)))
