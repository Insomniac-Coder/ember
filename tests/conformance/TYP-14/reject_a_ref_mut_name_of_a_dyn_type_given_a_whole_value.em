#$ test: compile-fail
#$ rules: TYP-14
#$ error[E3023]: `v` views a value of a kind known only while running; change it through its methods, `v.scale(2.0)`, or take the value over, `owns v: owns dyn Shape`, and return the new one
## `[TYP-14]` - `=` on a `ref mut` name of a `dyn I` type never gives the place a whole value.

interface Shape:
    fn scale(k: float)

struct Sq implements Shape:
    s: float

    fn scale(k: float):
        this.s *= k

fn grow(ref mut v: dyn Shape):
    v = Sq(1.0)

fn main():
    println(1)
