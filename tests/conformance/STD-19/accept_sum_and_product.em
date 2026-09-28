#$ test: run-pass
#$ rules: STD-19, STD-5
#$ profiles: debug, release, shipping
#$ stdout:
#$ 10 24 0 1
#$ 15 18 0.75
#$ 3 22
# `[STD-19]`, `[STD-5]` — `sum` and `product` combine an iterator's numbers,
# or the numbers its references reach, left to right in that number type:
# an empty sum is zero and an empty product one. A program's own iterator, a
# list's (of `int` and of `f64`), a map's values, and adapters over them.

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

fn main():
    println(Count(0, 4).sum(), Count(0, 4).product(), Count(0, 0).sum(), Count(0, 0).product())
    xs: Array[int] = [4, 9, 2]
    fs: Array[f64] = [0.5, 0.25]
    println(xs.iter().sum(), xs.iter().skip(1).product(), fs.iter().sum())
    m: Map[str, int] = {"a": 1, "b": 2}
    println(m.values().sum(), Count(0, 10).step_by(3).sum())
