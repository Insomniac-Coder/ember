#$ test: compile-fail
#$ rules: R14
## R14: "`cast s when Circle`, are not Ember (E1010 ...)"; verdict 27 removed verdict 25's spelling
## (AN-68).

interface Shape:
    fn size() -> int

struct Circle implements Shape:
    r: int

    fn size() -> int:
        return this.r

fn radius(s: dyn Shape) -> int:
    found = cast s when Circle    #$ error[E1010]: `cast s when Circle` is not Ember: instead of `found = cast s when Circle` and `if found exists as c:`, write `if s is Circle as c:`, keeping its block
    if found exists as c:
        return c.r
    return 0

fn main():
    println(radius(Circle(r=2)))
