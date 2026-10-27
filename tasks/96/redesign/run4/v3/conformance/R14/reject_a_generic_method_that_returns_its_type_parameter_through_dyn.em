#$ test: compile-fail
#$ rules: R14
## R14: only a generic method whose type parameter is only the type of a `ref` or `ref mut` parameter
## gets a `dyn` entry; one that returns its `V` makes the interface unusable as `dyn` (E2050, AN-67).

interface Visitor:
    fn visit(n: int) changes this

interface Inspect:
    [V: Visitor]
    fn make() -> V

fn walk(xs: Array[owns dyn Inspect]) -> int:    #$ error[E2050]: `Inspect` cannot be used as `dyn Inspect`: `make` returns its `V`, and a `dyn` value has one entry per method; take the visitor in instead of returning it, `[V: Visitor]` above `fn make(ref mut v: V)`, or keep each list's own type with a type parameter instead of `dyn Inspect`: `[I: Inspect]` above `fn walk(xs: Array[I]) -> int:`
    return xs.len()

fn main():
    println(walk([]))
