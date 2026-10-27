#$ test: run-pass
#$ rules: R5
#$ stdout: 0.0 0.0
## R5: on a struct, `changes this` lets a method change any field or the whole value
## (`this = Point(x=0.0, y=0.0)`; vocabulary, `this` row).

struct Point:
    x: float
    y: float

    fn reset() changes this:
        this = Point(x=0.0, y=0.0)

fn main():
    p = Point(x=1.0, y=2.0)
    p.reset()
    println(p.x, p.y)
