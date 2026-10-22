#$ test: run-pass
#$ rules: TYP-22, TYP-36, STR-5
#$ profiles: debug, development, release
#$ stdout: 4.0 6.0 1.0 0.5
#$ 4.0 6.0 1.0 0.5
#$ 9.0 4.0
## `[TYP-22]` (ODR-237) — an interface may require `Clone` (`interface Shape: Clone:`): every type
## implementing it is `Clone`, it is usable as `dyn`, and `Box[dyn Shape]` is `Clone`, its clone a
## new box holding a clone of the value, made by the entry `Shape`'s method table holds for the
## value's type: a struct, an enum, a scalar, and a class (whose clone is its handle, the object
## shared). So an array of them clones, and a struct with a `Box[dyn Shape]` field derives `Clone`.

interface Shape: Clone:
    fn area(self) -> float

struct Square:
    side: float
    name: String

enum Rect:
    Sized(float, float)

class Tile:
    size: float

    fn init(mut self, size: float):
        self.size = size

extend Square implements Shape:
    fn area(self) -> float:
        return self.side * self.side

extend Rect implements Shape:
    fn area(self) -> float:
        match self:
            Rect.Sized(w, h):
                return w * h

extend float implements Shape:
    fn area(self) -> float:
        return self

extend Tile implements Shape:
    fn area(self) -> float:
        return self.size

struct Scene:
    shape: Box[dyn Shape]

fn show(shapes: Span[Box[dyn Shape]]):
    out = String()
    for s in shapes:
        if not out.is_empty():
            out.push_str(" ")
        out.push_str(f"{s.area()}")
    println(out)

fn main():
    shapes: Array[Box[dyn Shape]] = [Box(Square(2.0, "a")), Box(Rect.Sized(2.0, 3.0)), Box(1.0), Box(Tile(0.5))]
    copies = shapes.clone()
    show(shapes.as_span())
    show(copies.as_span())
    scene = Scene(Box(Square(3.0, "b")))
    other = scene.clone()
    scene.shape = Box(Square(2.0, "c"))
    println(other.shape.area(), scene.shape.area())
