# A13: the dyn-compatible rewrite: the visitor passed as `dyn` instead of a type parameter.
interface Visitor:
    fn visit(mut self, n: int)

struct Sum implements Visitor:
    total: int
    fn visit(mut self, n: int):
        self.total += n

interface Inspect:
    fn inspect(self, v: ref mut dyn Visitor)

struct Leaf implements Inspect:
    n: int
    fn inspect(self, v: ref mut dyn Visitor):
        v.visit(self.n)

fn walk(xs: Array[Box[dyn Inspect]]) -> int:
    s = Sum(total=0)
    for x in xs:
        x.inspect(ref mut s)
    return s.total

fn main():
    println(walk([Box(Leaf(n=1)), Box(Leaf(n=2))]))
