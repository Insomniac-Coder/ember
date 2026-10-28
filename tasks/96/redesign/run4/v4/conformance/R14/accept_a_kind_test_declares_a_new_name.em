#$ test: run-pass
#$ rules: R14
#$ stdout: 2
## R14: "`if s is Circle as c:` ... names it `c` there. The name is new."

interface Shape:
    fn size() -> int

struct Circle implements Shape:
    r: int

    fn size() -> int:
        return this.r

fn radius(s: dyn Shape) -> int:
    if s is Circle as c:
        return c.r
    return 0

fn main():
    println(radius(Circle(r=2)))
