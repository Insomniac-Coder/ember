#$ test: run-pass
#$ rules: EFF-2, EFF-5
#$ profiles: debug, development, release
#$ stdout: 13
# `[EFF-2]` (A4-1) — a call through a `dyn` value runs one of the implementations the program has put
# behind that interface, so its effects are theirs: here neither allocates, and the `@noalloc`
# function calling through it is accepted. Before, such a call had every effect ("a call through a
# `dyn` value, whose effects nothing declares").

interface Shape:
    fn area(self) -> int

struct Square:
    side: int

struct Strip:
    long: int

extend Square implements Shape:
    fn area(self) -> int:
        return self.side * self.side

extend Strip implements Shape:
    fn area(self) -> int:
        return self.long

@noalloc
fn area_of(shape: ref dyn Shape) -> int:
    return shape.area()

fn main():
    square = Square(side = 3)
    strip = Strip(long = 4)
    println(area_of(ref square) + area_of(ref strip))
