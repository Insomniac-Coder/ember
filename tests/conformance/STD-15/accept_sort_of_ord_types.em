#$ test: run-pass
#$ rules: STD-15, TYP-17
#$ stdout:
#$ 1s 2d 3h 3c
#$ 1 3 2 1 3
#$ ['a', 'b'] ['b', 'a'] ['a', 'b']
# `[STD-15]` — `sort` takes any `T: Ord`, not only the numbers and text the
# compiler orders itself; it is stable (D-256). So does `sorted`, the method and
# `[STD-26]`'s function, for any `T: Ord + Clone`, `String` included.

from std.core import Ordering, Ord, Eq

struct Card:
    rank: int
    suit: String

extend Card implements Eq:
    fn eq(self, other: Self) -> bool:
        return self.rank == other.rank

extend Card implements Ord:
    fn cmp(self, other: Self) -> Ordering:
        return self.rank.cmp(other.rank)

@derive(Copy)
struct P:
    x: int

extend P implements Eq:
    fn eq(self, other: Self) -> bool:
        return self.x == other.x

extend P implements Ord:
    fn cmp(self, other: Self) -> Ordering:
        return self.x.cmp(other.x)

fn main():
    cards = [Card(3, String.from("h")), Card(1, String.from("s")), Card(3, String.from("c")), Card(2, String.from("d"))]
    cards.sort()
    shown: Array[String] = []
    for c in cards:
        shown.push(f"{c.rank}{c.suit}")
    println(shown.join(" "))

    ps = [P(2), P(1), P(3)]
    qs = ps.sorted()
    rs = sorted(ps)
    println(qs[0].x, qs[2].x, ps[0].x, rs[0].x, rs[2].x)

    words = [String.from("b"), String.from("a")]
    println(words.sorted(), words, sorted(words))
