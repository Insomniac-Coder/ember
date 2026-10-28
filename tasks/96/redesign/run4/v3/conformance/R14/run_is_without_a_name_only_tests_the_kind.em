#$ test: run-pass
#$ rules: R14
#$ stdout: 2
## R14: "`if s is Circle:` tests without a name"; one type-id compare (section 2).

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
    shapes: Array[owns dyn Shape] = [Circle(r=1), Square(side=2), Circle(r=3)]
    println(count_circles(shapes))
