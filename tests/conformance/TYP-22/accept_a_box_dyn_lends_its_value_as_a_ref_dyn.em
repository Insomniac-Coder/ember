#$ test: run-pass
#$ rules: TYP-22
#$ profiles: debug, development, release
#$ stdout: 4 9 4
#$ stdout: 7 8
#$ stdout: 5
# `[TYP-22]` (D-908) — a `Box[dyn I]` given to a `ref dyn I` parameter lends the box's own data and
# table: a struct concrete, a class concrete (the object shared with another handle, whose change
# the next call sees) and a box held in a field (the LLVM backend's port of D-908, tried against the
# C backend's).

interface Shape:
    fn area(self) -> int

struct Square:
    side: int

extend Square implements Shape:
    fn area(self) -> int:
        return self.side * self.side

class Tile:
    size: int

    fn init(mut self, size: int):
        self.size = size

    fn grow(mut self):
        self.size = self.size + 1

extend Tile implements Shape:
    fn area(self) -> int:
        return self.size

fn measure(s: ref dyn Shape) -> int:
    return s.area()

struct Holder:
    boxed: Box[dyn Shape]

fn main():
    a: Box[dyn Shape] = Box(Square(2))
    b: Box[dyn Shape] = Box(Square(3))
    println(measure(a), measure(b), a.area())
    tile = Tile(7)
    keep = tile
    t: Box[dyn Shape] = Box(tile)
    m = measure(t)
    keep.grow()
    println(m, measure(t))
    h = Holder(Box(Square(1)))
    h.boxed = Box(Tile(5))
    println(measure(h.boxed))
