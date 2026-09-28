#$ test: run-pass
#$ rules: STD-19, STD-26
#$ profiles: debug, release, shipping
#$ stdout:
#$ [1, 2, 3] [3, 4, 5] [1, 4, 7, 10]
#$ [(0, 1), (1, 2), (2, 3)] [(1, 11), (2, 12), (3, 13)]
#$ 5 Some(5) Some(5) None
#$ 10 true false Some(4) Some(5)
#$ [(10, 2), (11, 3)]
#$ 5 2 35 true Some(2)
#$ [11, 13, 15, 17]
#$ 3 6 2
#$ 123
# `[STD-19]` (ODR-089) — every `Iterator` has the adapters `take`, `skip`,
# `step_by`, `enumerate` (from `start`) and `zip`, and the consumers `count`,
# `last`, `nth`, `fold`, `any`, `all`, `find`, `position`, `for_each` and
# `to_array`: a program's own iterator, a list's view iterator (its items are
# references), and a map's and a set's.

from std.core import Iterator

struct Count:
    n: int

extend Count implements Iterator:
    type Item = int

    fn next(mut self) -> Option[int]:
        self.n += 1
        return Some(self.n)

fn main():
    println(Count(0).take(3).to_array(), Count(0).skip(2).take(3).to_array(), Count(0).step_by(3).take(4).to_array())
    println(Count(0).take(3).enumerate().to_array(), Count(0).take(3).zip(Count(10)).to_array())
    println(Count(0).take(5).count(), Count(0).take(5).last(), Count(0).nth(4), Count(0).take(2).nth(2))
    println(Count(0).take(4).fold(0, fn(acc, x) => acc + x), Count(0).take(4).any(fn(x) => x > 3), Count(0).take(4).all(fn(x) => x > 3), Count(0).find(fn(x) => x % 4 == 0), Count(0).position(fn(x) => x == 6))
    numbered: Array[(int, int)] = []
    for (i, x) in Count(0).skip(1).take(2).enumerate(10):
        numbered.push((i, x))
    println(numbered)

    xs: Array[int] = [5, 6, 7, 8, 9]
    println(xs.iter().count(), xs.iter().skip(1).step_by(2).count(), xs.iter().fold(0, fn(acc, x) => acc + x), xs.iter().any(fn(x) => x > 8), xs.iter().position(fn(x) => x == 7))
    sums: Array[int] = []
    for (a, b) in xs.iter().zip(xs.iter().skip(1)):
        sums.push(a + b)
    println(sums)

    m: Map[str, int] = {"a": 1, "b": 2, "c": 3}
    s: Set[int] = {4, 5}
    println(m.keys().count(), m.values().fold(0, fn(acc, v) => acc + v), s.iter().count())
    Count(0).take(3).for_each(fn(x) => print(x))
    println()
