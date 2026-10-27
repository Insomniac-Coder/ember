#$ test: run-pass
#$ rules: R14, R3
#$ stdout: 1
#$ stdout: 2
#$ stdout: 7
#$ stdout: 7
## R3 and R14: `owns dyn Shape` (the list is each shape's one owner) and `shares dyn Shape` (one of
## several owners, reading it); `dyn Shape` as a parameter's type is a read-only view (AN-66's NEW,
## made a whole program).

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
    scene = Scene(shapes=[Circle(r=1), Square(side=2)], shown=[shares c])
    for s in scene.shapes:
        draw(s)
    for s in scene.shown:
        draw(s)
    println(c.r)
