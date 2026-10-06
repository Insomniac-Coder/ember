#$ test: run-pass
#$ rules: IFC-3, TYP-17
#$ profiles: debug, release, shipping
#$ stdout: 9 square
# `[IFC-3]` (D-532) — an associated type's bound brings its parents, as a parameter's does:
# `Thing: Named` is a `Shape` too, so `h.get().area()` needs no bound of its own.

interface Shape:
    fn area(self) -> int

interface Named: Shape:
    fn name(self) -> str

interface Holder:
    type Thing: Named
    fn get(self) -> Thing

@derive(Copy)
struct Square:
    side: int

extend Square implements Shape:
    fn area(self) -> int:
        return self.side * self.side

extend Square implements Named:
    fn name(self) -> str:
        return "square"

struct Crate:
    inside: Square

extend Crate implements Holder:
    type Thing = Square

    fn get(self) -> Square:
        return self.inside

fn area_inside[H: Holder](h: H) -> int:
    return h.get().area()

fn main():
    c = Crate(Square(3))
    println(area_inside(c), c.get().name())
