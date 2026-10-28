#$ test: run-pass
#$ rules: TYP-44
#$ stdout: 5 0 5
## design rules: R14
## R14: the lines the E1010 messages give: the kind test for "not sure" (`if s is Circle as c:`) and
## `cast s as Circle` for "sure" (verdict 27).

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

fn maybe(s: dyn Shape) -> int:
    if s is Circle as c:
        return c.r
    return 0

fn sure(s: dyn Shape) -> int:
    a = cast s as Circle
    return a.r

fn main():
    c = Circle(r=5)
    q = Square(side=2)
    println(maybe(c), maybe(q), sure(c))
