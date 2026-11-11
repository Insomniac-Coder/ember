#$ test: run-pass
#$ rules: OBJ-2
#$ stdout: 4 9
# `[OBJ-2]` — `dyn I` as a parameter's type is a read-only view of any value that implements `I`.

interface Shape:
    fn area() -> int

struct Square:
    s: int

extend Square implements Shape:
    fn area() -> int:
        return this.s * this.s

class Tile implements Shape:
    w: int

    fn area() -> int:
        return this.w * this.w

fn show(s: dyn Shape) -> int:
    return s.area()

fn main():
    sq = Square(s=2)
    tile = Tile(w=3)
    println(show(ref sq), show(ref tile))
