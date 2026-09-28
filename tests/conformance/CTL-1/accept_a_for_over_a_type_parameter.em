#$ test: run-pass
#$ rules: CTL-1, TYP-17, IFC-4
#$ profiles: debug, release, shipping
#$ stdout: 3 15 | 2 6
# D-382 — `for x in it` where `it`'s type is a parameter bounded by `Iterator`
# takes `next` from the bound, as `it.next()` does; `x` is that parameter's
# `Item`.

from std.core import Iterator

struct Count:
    n: int
    limit: int

extend Count implements Iterator:
    type Item = int

    fn next(mut self) -> Option[int]:
        if self.n >= self.limit:
            return None
        self.n += 1
        return Some(self.n)

fn tally[I: Iterator](owned it: I) -> int:
    n = 0
    for _ in it:
        n += 1
    return n

fn sum_of[I: Iterator[Item = int]](owned it: I) -> int:
    s = 0
    for x in it:
        s += x
    return s

fn main():
    xs: Array[int] = [5, 6]
    println(tally(Count(0, 3)), sum_of(Count(0, 5)), "|", tally(xs.iter()), sum_of(Count(5, 6)))
