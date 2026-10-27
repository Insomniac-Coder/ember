#$ test: compile-fail
#$ rules: TYP-45, IFC-5
#$ help: say who owns the shapes: write `shapes: Array[owns dyn Shape]` in place of line 19 (the list is their only owner), and `scene = Scene(shapes=[owns Square(side=2)])` in place of line 22
## design rules: R3, R14
## R3, R14: a bare interface name used as a type is refused (E2173): the line must say who owns the
## shapes (AN-66's message); the way out gives the making line too, which then says `owns` (verdict 85),
## so the fix compiles (verdict 73).

interface Shape:
    fn area() -> int

struct Square:
    side: int

extend Square implements Shape:
    fn area() -> int:
        return this.side * this.side

struct Scene:
    shapes: Array[Shape]       #$ error[E2173]: `Shape` is an interface, so `Array[Shape]` does not say who owns the shapes; say who owns them: write `shapes: Array[owns dyn Shape]` in place of line 19 (the list is their only owner), and `scene = Scene(shapes=[owns Square(side=2)])` in place of line 22

fn main():
    scene = Scene(shapes=[Square(side=2)])
