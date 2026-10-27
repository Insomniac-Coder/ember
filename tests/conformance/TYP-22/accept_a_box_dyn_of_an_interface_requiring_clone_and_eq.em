#$ test: run-pass
#$ rules: TYP-22, TYP-36, DSP-3
#$ profiles: debug, development, release
#$ stdout: 6 12 4 true false
#$ stdout: 7 3 true true false
#$ stdout: 6 true
# `[TYP-22]`, IX.1 (ODR-237, ODR-241) — an interface requiring both `Clone` and `Eq`: its tables
# carry the `clone` entry, then the `eq` entry, then the interface's own methods, so a call of the
# third method reaches the right slot; a class concrete compares through its written `eq` and
# clones as its handle, and an `Array` of such boxes clones and compares element by element (the
# LLVM backend's port of D-909, tried against the C backend's).

interface Shape: Clone + Eq:
    fn area(self) -> int
    fn sides(self) -> int
    fn scaled(self, k: int) -> int

@derive(Eq)
struct Square:
    side: int

extend Square implements Shape:
    fn area(self) -> int:
        return self.side * self.side

    fn sides(self) -> int:
        return 4

    fn scaled(self, k: int) -> int:
        return self.side * k

class Tile:
    size: int

    fn init(mut self, size: int):
        self.size = size

    fn eq(self, other: Tile) -> bool:
        return self.size == other.size

extend Tile implements Shape:
    fn area(self) -> int:
        return self.size

    fn sides(self) -> int:
        return 3

    fn scaled(self, k: int) -> int:
        return self.size + k

fn main():
    a: Box[dyn Shape] = Box(Square(2))
    b = a.clone()
    t: Box[dyn Shape] = Box(Tile(5))
    println(a.scaled(3), b.scaled(6), b.sides(), a == b, a == t)
    u: Box[dyn Shape] = Box(Tile(5))
    v: Box[dyn Shape] = Box(Tile(6))
    w = t.clone()
    println(w.scaled(2), w.sides(), t == u, w == t, t == v)
    shapes: Array[Box[dyn Shape]] = [a.clone(), t.clone()]
    copies = shapes.clone()
    println(copies[1].scaled(1), copies == shapes)
