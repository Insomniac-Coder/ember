#$ test: run-pass
#$ rules: MONO-6, MONO-3
#$ assert-c: !contains("ember_dyn _1")
#$ stdout: 4.0 3.0
## `[MONO-6]` — sharing is the compiler's to choose only for a shareable generic past the
## `[MONO-3]` ceiling, none of whose calls is in a hot loop, and only `@never_specialize` asks for it
## here: `area_of` has two instances, one more than this package's ceiling (`W2220`), and each is
## still its own specialised body.

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

fn area_of[T: Shape](x: T) -> f64:  #$ warning[W2220]: `area_of` has 2 instances, more than `max_instantiations = 1`
    return x.area()

fn main():
    println(area_of(Square(2.0)), area_of(Disc(1.0)))
