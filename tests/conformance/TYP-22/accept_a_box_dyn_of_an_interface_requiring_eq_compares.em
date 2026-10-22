#$ test: run-pass
#$ rules: TYP-22, TYP-36, DSP-3
#$ profiles: debug, development, release
#$ stdout: true false true true
#$ stdout: true false
#$ stdout: 4 4
# `[TYP-22]`, IX.1 (ODR-241) — `Box[dyn I]` compares with `==` when `I` requires `Eq`
# (`interface Shape: Eq:`), through an entry of its method table as `Clone` is (ODR-237): two of
# different kinds are never equal, two of one kind compare their contents as that type writes
# `==` (`Loose`'s written `eq`). An `Array` of them compares element by element, and the table
# still dispatches the interface's own methods.

interface Shape: Eq:
    fn area(self) -> int

@derive(Eq)
struct Square:
    side: int

extend Square implements Shape:
    fn area(self) -> int:
        return self.side * self.side

## Equal when the last digits are.
struct Loose:
    n: int

    fn eq(self, other: Loose) -> bool:
        return self.n % 10 == other.n % 10

extend Loose implements Shape:
    fn area(self) -> int:
        return self.n

fn main():
    a: Box[dyn Shape] = Box(Square(2))
    b: Box[dyn Shape] = Box(Square(2))
    c: Box[dyn Shape] = Box(Loose(4))
    d: Box[dyn Shape] = Box(Loose(14))
    println(a == b, a == c, c == d, a != c)
    shapes: Array[Box[dyn Shape]] = [Box(Square(2)), Box(Loose(4))]
    same: Array[Box[dyn Shape]] = [Box(Square(2)), Box(Loose(24))]
    swapped: Array[Box[dyn Shape]] = [Box(Loose(4)), Box(Square(2))]
    println(shapes == same, shapes == swapped)
    println(a.area(), c.area())
