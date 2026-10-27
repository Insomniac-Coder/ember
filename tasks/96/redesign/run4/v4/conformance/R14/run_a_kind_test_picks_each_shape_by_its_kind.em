#$ test: run-pass
#$ rules: R14
#$ stdout: circle 3
#$ stdout: square 4
#$ stdout: other
## R14: "`if s is Circle as c:` runs its block when `s` is a `Circle` and names it `c` there ... Other
## kinds are tested with `elif s is Square as q:` and `else:`"; each test is one type-id compare
## (section 2). AN-68's NEW, made a whole program.

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

struct Dot implements Shape:
    x: int

    fn size() -> int:
        return 0

fn describe(s: dyn Shape):
    if s is Circle as c:
        println("circle", c.r)
    elif s is Square as q:
        println("square", q.side)
    else:
        println("other")

fn main():
    shapes: Array[owns dyn Shape] = [Circle(r=3), Square(side=4), Dot(x=0)]
    for s in shapes:
        describe(s)
