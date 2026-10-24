# A13: an interface whose method is generic, used through `dyn` (Rust: E0038).
interface Visitor:
    fn visit(mut self, n: int)

struct Sum implements Visitor:
    total: int
    fn visit(mut self, n: int):
        self.total += n

interface Inspect:
    fn inspect[V: Visitor](self, mut v: V)

struct Leaf implements Inspect:
    n: int
    fn inspect[V: Visitor](self, mut v: V):
        v.visit(self.n)

fn walk(xs: Array[Box[dyn Inspect]]) -> int:
    s = Sum(total=0)
    for x in xs:
        x.inspect(s)
    return s.total

fn main():
    println(walk([Box(Leaf(n=1)), Box(Leaf(n=2))]))
