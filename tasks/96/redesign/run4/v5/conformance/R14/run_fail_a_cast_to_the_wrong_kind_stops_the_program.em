#$ test: run-fail
#$ rules: R14
#$ stdout: 3
#$ panics: `cast b as Circle` on line 29: `b` is a `Square`, not a `Circle`
## R14: "`cast s as Circle` gives the `Circle` and stops the program if `s` is not one (section 2 gives
## the text)"; one type-id compare per cast; the stop's text is section 2's, in this program's names.
## A value made for an `owns dyn Shape` place says `owns` (verdict 85).

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

fn main():
    a: owns dyn Shape = owns Circle(r=3)
    c = cast a as Circle
    println(c.r)
    b: owns dyn Shape = owns Square(side=4)
    q = cast b as Circle
    println(q.r)
