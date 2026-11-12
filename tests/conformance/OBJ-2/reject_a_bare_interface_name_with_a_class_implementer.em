#$ test: compile-fail
#$ rules: OBJ-2, IFC-5, TYP-45
# `[OBJ-2]` - a bare interface name in a type position does not say who owns the value, whatever
# implements it: with a class implementer it is `E2173` as it is with a struct (`Array[Shape]` was
# accepted as a list of class handles).

interface Shape:
    fn area() -> int

class Square implements Shape:
    side: int

    fn area() -> int:
        return this.side * this.side

fn main():
    shapes: Array[Shape] = []       #$ error[E2173]: `Shape` is an interface, so `Array[Shape]` does not say who owns the shapes; say who owns them: write `shapes: Array[owns dyn Shape]` in place of line 17 (the list is their only owner)
    println(shapes.len())
