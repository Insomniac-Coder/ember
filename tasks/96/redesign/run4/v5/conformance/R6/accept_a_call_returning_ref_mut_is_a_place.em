#$ test: run-pass
#$ rules: LT-1, BRW-12
#$ stdout: 2 1
#$ stdout: 12
## design rules: R6, R2
## R6: a call whose result type is `ref mut T` is a place: `first(ref mut xs) += 1` writes `xs[0]`,
## `counts.entry("a").or_default() += 1` writes the map's count, and a name bound to such a call says
## `ref mut` (R2) and writes the element (AN-23).

from std.collections import Map

fn first(ref mut xs: Array[int]) -> ref mut int:
    return ref mut xs[0]

fn main():
    xs = [1, 2, 3]
    first(ref mut xs) += 1
    counts = Map[String, int]()
    counts.entry("a").or_default() += 1
    println(xs[0], counts["a"])
    f = ref mut first(ref mut xs)
    f = 12
    println(xs[0])
