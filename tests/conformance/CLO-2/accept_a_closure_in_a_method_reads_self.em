#$ test: run-pass
#$ rules: CLO-2, CLS-2
#$ stdout: 30 8
#$ stdout: 12
#$ stdout: 8
# D-566: a closure in a method captures its receiver as it captures any other
# local of the method; in a class `init` it needs every field first.

class Counter:
    n: int = 3

    fn total() -> int:
        f = fn() -> int => this.n * 10
        return f()

    fn later() -> int:
        g = fn(k: int) -> int => this.n + k
        return g(5)

struct Pt:
    x: int

    fn scaled(k: int) -> int:
        f = fn() -> int => this.x * k
        return f()

class Named:
    name: String
    size: int

    fn init(owns name: String):
        this.name = name
        this.size = 2
        measure = fn() -> int => this.name.len() * this.size
        println(measure())

fn main():
    c = Counter()
    println(c.total(), c.later())
    println(Pt(4).scaled(3))
    _n = Named("abcd")
