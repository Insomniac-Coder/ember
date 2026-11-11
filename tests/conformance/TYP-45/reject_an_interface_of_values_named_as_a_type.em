#$ test: compile-fail
#$ rules: TYP-45, GRM-8b
## `[TYP-45]`, `[GRM-8b]` — an interface named bare as a type does not say who owns its values:
## `E2173`. When every type that implements it is a value, not a class, the help names the ways
## that do (`owns dyn Shape`, `ref dyn Shape`, or a type parameter).

interface Shape:
    fn area() -> int

struct Square:
    side: int

extend Square implements Shape:
    fn area() -> int:
        return this.side * this.side

fn main():
    shapes: Array[Shape] = []    #$ error[E2173]: `Shape` is an interface, so `Array[Shape]` does not say who owns the shapes; say who owns them: write `shapes: Array[owns dyn Shape]` in place of line 18 (the list is their only owner)
    println(shapes.len())
