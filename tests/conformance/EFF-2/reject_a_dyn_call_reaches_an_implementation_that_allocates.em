#$ test: compile-fail
#$ rules: EFF-2, EFF-5
# `[EFF-2]` (A4-1) — a call through a `dyn` value has the effects of every implementation the program
# has put behind the interface: one of them allocates, so the `@noalloc` function calling through it
# reaches that allocation.

interface Shape:
    fn area(self) -> int

struct Square:
    side: int

struct Ring:
    sides: int

extend Square implements Shape:
    fn area(self) -> int:
        return self.side * self.side

extend Ring implements Shape:
    fn area(self) -> int:
        return [self.sides, self.sides].len()

@noalloc
fn area_of(shape: ref dyn Shape) -> int:
    return shape.area()    #$ error[E4001]: `@noalloc` function `area_of` reaches an allocation

fn main():
    square = Square(side = 3)
    ring = Ring(sides = 6)
    println(area_of(ref square) + area_of(ref ring))
