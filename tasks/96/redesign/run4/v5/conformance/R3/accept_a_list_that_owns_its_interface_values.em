#$ test: run-pass
#$ rules: OWN-8, TYP-44
#$ stdout: 13
## design rules: R3, R14
## R3, R14: the nearest program that is allowed: `owns dyn Shape` says the list is each shape's one
## owner, and the line that makes each shape says so, `owns Square(...)`, as for an `owns` field
## (verdict 85); each is put in its own block (AN-66's NEW).

interface Shape:
    fn area() -> int

struct Square:
    side: int

extend Square implements Shape:
    fn area() -> int:
        return this.side * this.side

struct Scene:
    shapes: Array[owns dyn Shape]

fn main():
    scene = Scene(shapes=[owns Square(side=2), owns Square(side=3)])
    total = 0
    for s in scene.shapes:
        total += s.area()
    println(total)
