#$ test: compile-fail
#$ rules: OBJ-2, IFC-5
# `[OBJ-2]` — a bare interface name in a type position (`s: Shape`, `Array[Shape]`) does not say who
# owns the value, and is `E2173`: the message names the written type, the place, its line, and the
# type with `owns dyn`.

interface Shape:
    fn area() -> int

struct Square:
    s: int

extend Square implements Shape:
    fn area() -> int:
        return this.s

fn main():
    shapes: Array[Shape] = []       #$ error[E2173]: `Shape` is an interface, so `Array[Shape]` does not say who owns the shapes; say who owns them: write `shapes: Array[owns dyn Shape]` in place of line 18 (the list is their only owner)
    first: Shape = Square(s=1)      #$ error[E2173]: `Shape` is an interface, so `Shape` does not say who owns the shape; say who owns it: write `first: owns dyn Shape` in place of line 19
    println(shapes.len())
