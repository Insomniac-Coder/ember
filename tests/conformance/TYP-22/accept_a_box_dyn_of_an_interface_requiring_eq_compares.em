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

interface Shape: needs Eq:
    fn area() -> int

@derive(Eq)
struct Square:
    side: int

extend Square implements Shape:
    fn area() -> int:
        return this.side * this.side

## Equal when the last digits are.
struct Loose:
    n: int

    fn eq(other: Loose) -> bool:
        return this.n % 10 == other.n % 10

extend Loose implements Shape:
    fn area() -> int:
        return this.n

fn main():
    a: owns dyn Shape = owns Square(2)
    b: owns dyn Shape = owns Square(2)
    c: owns dyn Shape = owns Loose(4)
    d: owns dyn Shape = owns Loose(14)
    println(a == b, a == c, c == d, a != c)
    shapes: Array[owns dyn Shape] = [owns Square(2), owns Loose(4)]
    same: Array[owns dyn Shape] = [owns Square(2), owns Loose(24)]
    swapped: Array[owns dyn Shape] = [owns Loose(4), owns Square(2)]
    println(shapes == same, shapes == swapped)
    println(a.area(), c.area())
