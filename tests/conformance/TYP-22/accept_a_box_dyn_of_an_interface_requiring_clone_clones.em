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

interface Shape: needs Clone:
    fn area() -> float

struct Square:
    side: float
    name: String

enum Rect:
    Sized(float, float)

class Tile:
    size: float

    fn init(size: float):
        this.size = size

extend Square implements Shape:
    fn area() -> float:
        return this.side * this.side

extend Rect implements Shape:
    fn area() -> float:
        match this:
            Rect.Sized(w, h):
                return w * h

extend float implements Shape:
    fn area() -> float:
        return this

extend Tile implements Shape:
    fn area() -> float:
        return this.size

struct Scene:
    shape: owns dyn Shape

fn show(shapes: Span[owns dyn Shape]):
    out = String()
    for s in shapes:
        if not out.is_empty():
            out.push_str(" ")
        out.push_str(f"{s.area()}")
    println(out)

fn main():
    shapes: Array[owns dyn Shape] = [owns Square(2.0, "a"), owns Rect.Sized(2.0, 3.0), owns 1.0, owns Tile(0.5)]
    copies = shapes.clone()
    show(shapes.as_span())
    show(copies.as_span())
    scene = Scene(owns Square(3.0, "b"))
    other = scene.clone()
    scene.shape = owns Square(2.0, "c")
    println(other.shape.area(), scene.shape.area())
