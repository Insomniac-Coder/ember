#$ test: run-pass
#$ rules: STD-19
#$ stdout: [3, 1, 3, 2, 1, 3]
#$ stdout: [30, 10, 30, 20, 10, 30]
#$ stdout: 3 true
#$ stdout: 30 20 2
#$ stdout: hey
#$ stdout: abcd
#$ stdout: 3 2 1
# ODR-094 — `collect[C]()` builds any `C: FromIterator[Item]`, written or the
# type the call is expected to give: `Array`, `Set`, `Map` from pairs (a later
# pair's value replacing an earlier one's), `String` from characters and from
# strings, and a program's own type.

from std.core import FromIterator

## How many of each value it was given.
struct Tally:
    counts: Map[int, int]

extend Tally implements FromIterator[int]:
    [I: needs Iterator[Item = int]]
    static fn from_iter(owns it: I) -> Tally:
        counts: Map[int, int] = Map()
        for x in it:
            if x in counts:
                counts[x] += 1
            else:
                counts[x] = 1
        return Tally(counts = counts)

fn main():
    xs = [3, 1, 3, 2, 1, 3]
    a = xs.iter().copy_each().collect[Array[int]]()
    println(a)
    b: Array[int] = xs.iter().copy_each().map(fn(x: int) => x * 10).collect()
    println(b)
    s: Set[int] = xs.iter().copy_each().collect()
    println(s.len(), s.contains(2))
    pairs = [(1, 10), (2, 20), (1, 30)]
    m: Map[int, int] = pairs.iter().copy_each().collect()
    println(m[1], m[2], m.len())
    cs = ['h', 'e', 'y']
    word: String = cs.iter().copy_each().collect()
    println(word)
    parts = ["ab".to_string(), "cd".to_string()]
    joined: String = parts.iter().copy_each().collect()
    println(joined)
    t: Tally = xs.iter().copy_each().collect()
    println(t.counts[3], t.counts[1], t.counts[2])
