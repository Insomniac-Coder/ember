#$ test: run-pass
#$ rules: R5
#$ stdout: 0.0 0.0
## R5: a struct method may change any field or replace the whole value (`this = Point(x=0.0, y=0.0)`;
## vocabulary, `this` row), with no word (verdict 86).

struct Point:
    x: float
    y: float

    fn reset():
        this = Point(x=0.0, y=0.0)

fn main():
    p = Point(x=1.0, y=2.0)
    p.reset()
    println(p.x, p.y)
