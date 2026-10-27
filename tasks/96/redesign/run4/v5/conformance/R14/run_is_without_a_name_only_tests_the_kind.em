#$ test: run-pass
#$ rules: R14
#$ stdout: 2
## R14: "`if s is Circle:` tests without a name"; one type-id compare (section 2). Each shape made for
## the list of `owns dyn Shape` says `owns` (verdict 85).

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

fn count_circles(shapes: Array[owns dyn Shape]) -> int:
    n = 0
    for s in shapes:
        if s is Circle:
            n += 1
    return n

fn main():
    shapes: Array[owns dyn Shape] = [owns Circle(r=1), owns Square(side=2), owns Circle(r=3)]
    println(count_circles(shapes))
