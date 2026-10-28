#$ test: run-pass
#$ rules: TYP-44, OWN-8
#$ stdout: 1
#$ stdout: 2
#$ stdout: 7
#$ stdout: 7
## design rules: R14, R3
## R3 and R14: `owns dyn Shape` (the list is each shape's one owner) and `shares dyn Shape` (one of
## several owners, reading it); `dyn Shape` as a parameter's type is a read-only view (AN-66's NEW,
## made a whole program). Each shape made for the list of `owns dyn Shape` says `owns` (verdict 85;
## R3: `shapes=[owns Circle(r=1.0), owns Square(side=2.0)]`).

interface Shape:
    fn size() -> int

struct Circle implements Shape:
    r: int

    fn size() -> int:
        return this.r

struct Square implements Shape:
    side: int

    fn size() -> int:
        return this.side

struct Scene:
    shapes: Array[owns dyn Shape]
    shown: Array[shares dyn Shape]

fn draw(s: dyn Shape):
    println(s.size())

fn main():
    c = shares Circle(r=7)
    scene = Scene(shapes=[owns Circle(r=1), owns Square(side=2)], shown=[shares c])
    for s in scene.shapes:
        draw(s)
    for s in scene.shown:
        draw(s)
    println(c.r)
