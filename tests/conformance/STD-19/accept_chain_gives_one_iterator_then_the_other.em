#$ test: run-pass
#$ rules: STD-19
#$ profiles: debug, release, shipping
#$ stdout:
#$ [1, 2, 3, 10, 20]
#$ 18
#$ 54
#$ [3, 10, 20]
#$ 2
#$ 2
#$ 0
#$ [(4, 20), (5, 1), (6, 2)]
#$ 84
# `[STD-19]` — `chain(other)` gives this iterator's items, then `other`'s,
# which are of the same type. Once the first has said `None` it is not asked
# again (`Once` panics if it is), and adapters and consumers go on after it.

struct Once:
    given: bool
    ended: bool

extend Once implements Iterator:
    type Item = int

    fn next(mut self) -> Option[int]:
        if self.ended:
            panic("asked again after `None`")
        if self.given:
            self.ended = true
            return None
        self.given = true
        return Some(3)

fn main():
    xs: Array[int] = [1, 2, 3]
    ys: Array[int] = [10, 20]
    empty: Array[int] = []
    println(xs.iter().copied().chain(ys.iter().copied()).to_array())
    total = 0
    for x in (0..3).iter().chain((7..9).iter()):
        total += x
    println(total)
    for x in xs.iter().chain(ys.iter()):
        total += x
    println(total)
    println(Once(false, false).chain(ys.iter().copied()).to_array())
    println(empty.iter().chain(ys.iter()).count())
    println(ys.iter().chain(empty.iter()).count())
    println(empty.iter().chain(empty.iter()).count())
    both = xs.iter().copied().chain(ys.iter().copied()).chain(xs.iter().copied())
    println(both.enumerate().skip(4).take(3).to_array())
    println(xs.iter().copied().chain(ys.iter().copied()).fold(0, fn(a, b) => a * 2 + b))
