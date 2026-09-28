#$ test: run-pass
#$ rules: STD-15
#$ profiles: debug, release, shipping
#$ assert-c-count: contains("EMBER_SORT_STABLE(") == 1
#$ assert-c-count: contains("EMBER_SORT_UNSTABLE(") == 1
#$ stdout:
#$ true 0 197
#$ true w0 w99
#$ true 48460 0 99
#$ true true true
# `[STD-15]` — `sort` is stable. `Card` has an `Ord` of its own (by rank
# only), so std's merge sort orders it: 200 cards span several runs and each
# rank's cards keep the order they came in. The compiler sorts numbers and
# text itself, specialised to the element type: text by a stable merge sort
# (200 strings, past one run of 32), and `int`, where equal means the same
# value so no order among equals can be seen, by introsort (1000 values with
# many repeats, and sorted, reversed and all-equal input).

from std.core import Ordering, Ord, Eq

@derive(Copy)
struct Card:
    rank: int
    tag: int

extend Card implements Eq:
    fn eq(self, other: Self) -> bool:
        return self.rank == other.rank

extend Card implements Ord:
    fn cmp(self, other: Self) -> Ordering:
        return self.rank.cmp(other.rank)

fn ascending(xs: Array[int]) -> bool:
    for i in 1..xs.len():
        if xs[i - 1] > xs[i]:
            return false
    return true

fn main():
    cards: Array[Card] = []
    for i in 0..200:
        cards.push(Card((i * 7) % 5, i))
    cards.sort()
    kept = true
    for i in 1..cards.len():
        a = cards[i - 1]
        b = cards[i]
        if a.rank > b.rank or (a.rank == b.rank and a.tag > b.tag):
            kept = false
    println(kept, cards[0].tag, cards[199].tag)

    words: Array[String] = []
    for i in 0..200:
        words.push(f"w{(i * 37) % 200}")
    words.sort()
    ordered = true
    for i in 1..words.len():
        if words[i - 1] > words[i]:
            ordered = false
    println(ordered, words[0], words[199])

    xs: Array[int] = []
    seed = 12345
    total = 0
    for i in 0..1000:
        seed = (seed * 1103515245 + 12345) % 2147483648
        xs.push(seed % 100)
    xs.sort()
    for x in xs:
        total += x
    println(ascending(xs), total, xs[0], xs[999])

    up: Array[int] = []
    down: Array[int] = []
    same: Array[int] = []
    for i in 0..500:
        up.push(i)
        down.push(500 - i)
        same.push(7)
    up.sort()
    down.sort()
    same.sort()
    println(ascending(up), ascending(down), ascending(same))
