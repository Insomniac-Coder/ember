# Clause IFC-6.1: "A function that is not `pub` writes all of its type parameters' bounds or none.
# With none, they are worked out from its body"
#$ test: run-pass
#$ rules: IFC-6, TYP-17, MOD-9
#$ stdout: 9
#$ stdout: 7
#$ stdout: ab
# `heaviest` calls `weight`, which `Weighed` declares; `sum2` uses `+`, which `Add` gives; `twice`
# writes `copy of`, which `Clone` gives. None writes a bound.

interface Weighed:
    fn weight() -> int

struct Crate implements Weighed:
    w: int

    fn weight() -> int:
        return this.w

[T]
fn heaviest(xs: Span[T]) -> int:
    best = 0
    for x in xs:
        if x.weight() > best:
            best = x.weight()
    return best

[T]
fn sum2(a: T, b: T) -> T:
    return a + b

[T]
fn twice(x: T) -> (T, T):
    return (copy of x, copy of x)

fn main():
    println(heaviest([Crate(3), Crate(9)]))
    println(sum2(3, 4))
    pair = twice(String("a"))
    println(f"{pair.0}b")
