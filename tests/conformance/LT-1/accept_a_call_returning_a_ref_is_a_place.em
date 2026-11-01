#$ test: run-pass
#$ rules: LT-1, BRW-12
#$ stdout: [2, 2, 3]
#$ [2, 12, 3]
#$ 2
# `[LT-1]` — a call whose result type is `ref mut T` or `ref T` is a place: `first(ref mut xs) += 1`,
# `counts.entry(k).or_default() += 1`, and a name bound to one with `ref mut` writes what it views.

fn first(ref mut xs: Array[int]) -> ref mut int:
    return ref mut xs[0]

fn second(ref mut xs: Array[int]) -> ref mut int:
    return ref mut xs[1]

fn main():
    xs = [1, 2, 3]
    first(ref mut xs) += 1
    println(xs)
    r = ref mut second(ref mut xs)
    r += 10
    println(xs)
    counts: Map[StrView, int] = {}
    counts.entry("a").or_default() += 1
    counts.entry("a").or_default() += 1
    println(counts["a"])
