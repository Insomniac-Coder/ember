#$ test: compile-fail
#$ rules: R14
## R14: "Today's `cast s if Circle` ... are not Ember (E1010, with the line to write:
## `if s is Circle as c:`)" (AN-68's OLD line). Each shape made for the list of `owns dyn Shape` says
## `owns` (verdict 85).

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

fn total(shapes: Array[owns dyn Shape]) -> int:
    t = 0
    for s in shapes:
        found = cast s if Circle    #$ error[E1010]: `cast s if Circle` is not Ember: to run a block when `s` is a `Circle`, write `if s is Circle as c:` in place of `found = cast s if Circle` and `if found exists as c:` (lines 25 and 26), keeping the block; to stop the program when it is not one, write `c = cast s as Circle`
        if found exists as c:
            t += c.r
    return t

fn main():
    shapes: Array[owns dyn Shape] = [owns Circle(r=1), owns Square(side=2)]
    println(total(shapes))
