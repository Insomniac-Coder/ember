#$ test: run-pass
#$ rules: MONO-6, MONO-7
#$ assert-c: !contains("ember_dyn _1")
#$ stdout: 4.0 3.0 1.0 3.0
## `[MONO-6]` — with no ceiling set, nothing is shared: each instance of `area_of` is its own body.
## `[MONO-7]` — `@always_specialize` forbids sharing; neither attribute changes what the program
## computes.

interface Shape:
    fn area(self) -> f64

struct Square:
    side: f64

extend Square implements Shape:
    fn area(self) -> f64:
        return self.side * self.side

struct Disc:
    r: f64

extend Disc implements Shape:
    fn area(self) -> f64:
        return 3.0 * self.r * self.r

fn area_of[T: Shape](x: T) -> f64:
    return x.area()

@always_specialize
fn exact[T: Shape](x: T) -> f64:
    return x.area()

fn main():
    println(area_of(Square(2.0)), area_of(Disc(1.0)), exact(Square(1.0)), exact(Disc(1.0)))
