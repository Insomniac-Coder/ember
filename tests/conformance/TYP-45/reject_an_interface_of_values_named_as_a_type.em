#$ test: compile-fail
#$ rules: TYP-45, GRM-8b
## `[TYP-45]`, `[GRM-8b]` — an interface named bare as a type does not say who owns its values:
## `E2173`. When every type that implements it is a value, not a class, the help names the ways
## that do (`Box[dyn Shape]`, `ref dyn Shape`, or a type parameter).

interface Shape:
    fn area() -> int

struct Square:
    side: int

extend Square implements Shape:
    fn area() -> int:
        return this.side * this.side

fn main():
    shapes: Array[Shape] = []    #$ error[E2173]: `Shape` is an interface, and the types implementing it are values, not classes
    println(shapes.len())
