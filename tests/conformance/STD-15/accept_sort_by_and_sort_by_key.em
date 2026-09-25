#$ test: run-pass
#$ rules: STD-15, HASH-3, CLO-3
#$ stdout:
#$ [9, 7, 5, 3, 3, 2, 1]
#$ ['fig', 'pear', 'kiwi', 'apple']
#$ 3a 3b 1c 1d
#$ [8, 1, 6, 3]
#$ keys 4 2 8 6
#$ [8, 6, 4, 2]
#$ 7
#$ []
#$ [3, 2, 1]
#$ a bb ccc
#$ 2 1
# `[STD-15]` (ODR-031) — `sort_by(cmp)` and `sort_by_key(f)` are stable, take
# any element type (no element is copied), accept capturing closures, and call
# `key` once per element, in order. An inconsistent `cmp` may only permute the
# elements (`[HASH-3]`): the count and the sum survive it. A comparator can be
# passed on to `sort_by` by a function that takes one (D-257), and a key may be
# `String` or a type with its own `Ord`.

from std.core import Ordering, Ord, Eq

struct Tagged:
    rank: int
    tag: String

struct Rank:
    n: int

extend Rank implements Eq:
    fn eq(self, other: Self) -> bool:
        return self.n == other.n

extend Rank implements Ord:
    fn cmp(self, other: Self) -> Ordering:
        return other.n.cmp(self.n)

fn descending(mut xs: Array[int], cmp: fn(int, int) -> Ordering):
    xs.sort_by(cmp)

fn noted(v: int) -> int:
    print(f" {v}")
    return 0 - v

fn main():
    xs = [5, 3, 9, 1, 3, 7, 2]
    xs.sort_by(fn(a, b) => b.cmp(a))
    println(xs)

    words = [String.from("pear"), String.from("fig"), String.from("apple"), String.from("kiwi")]
    words.sort_by_key(fn(w) => w.len())
    println(words)

    items = [Tagged(1, String.from("c")), Tagged(3, String.from("a")), Tagged(1, String.from("d")), Tagged(3, String.from("b"))]
    items.sort_by(fn(a, b) => b.rank.cmp(a.rank))
    shown: Array[String] = []
    for t in items:
        shown.push(f"{t.rank}{t.tag}")
    println(shown.join(" "))

    limit = 4
    ys = [6, 3, 8, 1]
    ys.sort_by(fn(a, b) => (a % limit).cmp(b % limit))
    println(ys)

    zs = [4, 2, 8, 6]
    print("keys")
    zs.sort_by_key(noted)
    println("")
    println(zs)

    odd = [3, 1, 2, 1]
    odd.sort_by(fn(a, b) => Ordering.Less)
    total = 0
    for v in odd:
        total += v
    println(total)

    empty: Array[int] = []
    empty.sort_by(fn(a, b) => a.cmp(b))
    println(empty)

    passed = [3, 1, 2]
    descending(passed, fn(a, b) => b.cmp(a))
    println(passed)

    texts = [String.from("ccc"), String.from("a"), String.from("bb")]
    texts.sort_by_key(fn(s) => s.clone())
    println(texts.join(" "))

    ranks = [1, 2]
    ranks.sort_by_key(fn(v) => Rank(v))
    println(ranks[0], ranks[1])
