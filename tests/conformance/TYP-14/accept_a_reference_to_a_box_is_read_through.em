#$ test: run-pass
#$ rules: TYP-14, TYP-22
#$ profiles: debug, development, release
#$ stdout: 4.0 4.0
#$ stdout: 1.0
#$ stdout: 9.0
#$ stdout: 9.0
# `[TYP-14]` — a reference and a `Box[T]` are read through wherever a `T` is wanted, an argument to a
# borrowed parameter among them, a reference to a `Box` too: a `for` loop's `ref Box[dyn Shape]` is
# passed where `ref dyn Shape` is wanted (the first week's 16 holds its shapes so), and a
# `ref Box[Square]` where a `Square` is.

interface Shape:
    fn area(self) -> float

struct Square:
    side: float

extend Square implements Shape:
    fn area(self) -> float:
        return self.side * self.side

fn measure(s: ref dyn Shape) -> float:
    return s.area()

fn measure_square(s: Square) -> float:
    return s.area()

fn main():
    boxed: Box[dyn Shape] = Box(Square(side=2.0))
    held = ref boxed
    println(measure(boxed), measure(held))
    shapes: Array[Box[dyn Shape]] = [Box(Square(side=1.0))]
    for s in shapes:
        println(measure(s))
    square: Box[Square] = Box(Square(side=3.0))
    seen = ref square
    println(measure_square(square))
    println(measure_square(seen))
