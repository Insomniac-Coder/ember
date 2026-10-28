#$ test: compile-fail
#$ rules: R14
## R14: "Today's `cast s if Circle` ... are not Ember (E1010, with the line to write:
## `if s is Circle as c:`)" (AN-68's OLD line).

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
        if c = cast s if Circle:    #$ error[E1010]: `cast s if Circle` is not Ember: to run a block when `s` is a `Circle` and call it `c`, write `if s is Circle as c:`; to stop the program when it is not one, write `c = cast s as Circle`
            t += c.r
    return t

fn main():
    shapes: Array[owns dyn Shape] = [Circle(r=1), Square(side=2)]
    println(total(shapes))
