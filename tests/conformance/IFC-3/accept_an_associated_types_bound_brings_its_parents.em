#$ test: run-pass
#$ rules: IFC-3, TYP-17
#$ profiles: debug, development, release
#$ stdout: 9 square
# `[IFC-3]` (D-532) — an associated type's bound brings its parents, as a parameter's does:
# `Thing: Named` is a `Shape` too, so `h.get().area()` needs no bound of its own.

interface Shape:
    fn area() -> int

interface Named: needs Shape:
    fn name() -> StrView

interface Holder:
    type Thing: needs Named
    fn get() -> Thing

@derive(Copy)
struct Square:
    side: int

extend Square implements Shape:
    fn area() -> int:
        return this.side * this.side

extend Square implements Named:
    fn name() -> StrView:
        return "square"

struct Crate:
    inside: Square

extend Crate implements Holder:
    type Thing = Square

    fn get() -> Square:
        return this.inside

[H: needs Holder]
fn area_inside(h: H) -> int:
    return h.get().area()

fn main():
    c = Crate(Square(3))
    println(area_inside(c), c.get().name())
