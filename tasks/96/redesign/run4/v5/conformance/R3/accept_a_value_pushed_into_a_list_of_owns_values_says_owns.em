#$ test: run-pass
#$ rules: R3, R14
#$ stdout: 13
## R3, R14: the nearest program that is allowed: the line that makes each shape for an
## `Array[owns dyn Shape]` says `owns`, in the list it is made in and at a `push` (verdict 85).

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
    scene = Scene(shapes=[owns Square(side=2)])
    scene.shapes.push(owns Square(side=3))
    total = 0
    for s in scene.shapes:
        total += s.area()
    println(total)
