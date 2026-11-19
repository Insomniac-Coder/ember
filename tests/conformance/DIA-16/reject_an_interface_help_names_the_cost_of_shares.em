#$ test: compile-fail
#$ rules: DIA-16, OBJ-2
# `[DIA-16]` — `E2173`'s help offers `shares dyn Shape`, so it names what a `shares` owner costs.

interface Shape:
    fn area() -> i32

class Tile implements Shape:
    w: i32

    fn area() -> i32:
        return this.w

fn main():
    shapes: Array[Shape] = []   #$ error[E2173]: a `shares` or `shares mut` owner costs, while the program runs, one count per owner made or ended, and one compare with the count's maximum per owner made
    println(shapes.len())
