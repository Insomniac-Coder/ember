#$ test: compile-fail
#$ rules: R3, R14
#$ help: say who owns the shapes: `Array[owns dyn Shape]` (the list is their only owner) or `Array[shares dyn Shape]` (other names own them too)
## R3, R14: a bare interface name used as a type is refused (E2173): the line must say who owns the
## shapes (AN-66's message).

interface Shape:
    fn area() -> int

struct Square:
    side: int

extend Square implements Shape:
    fn area() -> int:
        return this.side * this.side

struct Scene:
    shapes: Array[Shape]       #$ error[E2173]: `Shape` is an interface, so `Array[Shape]` does not say who owns the shapes

fn main():
    scene = Scene(shapes=[Square(side=2)])
