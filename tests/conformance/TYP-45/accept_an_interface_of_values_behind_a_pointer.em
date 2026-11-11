#$ test: run-pass
#$ rules: TYP-45, GRM-8b
#$ profiles: debug, development
#$ stdout: 16
## `[TYP-45]` — a value of a type implementing an interface is held as `Box[dyn Shape]`, which says
## who owns it; the interface's bare name as a type does not.

interface Shape:
    fn area() -> int

struct Square:
    side: int

extend Square implements Shape:
    fn area() -> int:
        return this.side * this.side

fn main():
    shapes: Array[owns dyn Shape] = []
    shapes.push(owns Square(side=4))
    println(shapes[0].area())
