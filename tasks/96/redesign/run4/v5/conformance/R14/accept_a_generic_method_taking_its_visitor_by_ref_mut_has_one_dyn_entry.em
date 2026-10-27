#$ test: run-pass
#$ rules: R14
#$ stdout: 5
## R14: "A generic method whose type parameter is only the type of a `ref` or `ref mut` parameter
## gets one `dyn` entry" (AN-67's `Inspect` and `walk`, made a whole program). Each leaf made for the
## list of `owns dyn Inspect` says `owns` (verdict 85).

interface Visitor:
    fn visit(n: int)

struct Sum implements Visitor:
    total: int

    fn visit(n: int):
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
    xs: Array[owns dyn Inspect] = [owns Leaf(n=2), owns Leaf(n=3)]
    println(walk(xs))
