#$ test: run-pass
#$ rules: R14
#$ stdout: 5
## R14: "A generic method whose type parameter is only the type of a `ref` or `ref mut` parameter
## gets one `dyn` entry" (AN-67's `Inspect` and `walk`, made a whole program).

interface Visitor:
    fn visit(n: int) changes this

struct Sum implements Visitor:
    total: int

    fn visit(n: int) changes total:
        this.total += n

interface Inspect:
    [V: Visitor]
    fn inspect(ref mut v: V)

struct Leaf implements Inspect:
    n: int

    [V: Visitor]
    fn inspect(ref mut v: V):
        v.visit(this.n)

fn walk(xs: Array[owns dyn Inspect]) -> int:
    s = Sum(total=0)
    for x in xs:
        x.inspect(ref mut s)
    return s.total

fn main():
    xs: Array[owns dyn Inspect] = [Leaf(n=2), Leaf(n=3)]
    println(walk(xs))
