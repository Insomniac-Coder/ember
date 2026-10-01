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
    fn from_iter[I: Iterator[Item = int]](owned it: I) -> Tally:
        counts: Map[int, int] = Map()
        for x in it:
            if x in counts:
                counts[x] += 1
            else:
                counts[x] = 1
        return Tally(counts = counts)

fn main():
    xs = [3, 1, 3, 2, 1, 3]
    a = xs.iter().copied().collect[Array[int]]()
    println(a)
    b: Array[int] = xs.iter().copied().map(fn(x: int) => x * 10).collect()
    println(b)
    s: Set[int] = xs.iter().copied().collect()
    println(s.len(), s.contains(2))
    pairs = [(1, 10), (2, 20), (1, 30)]
    m: Map[int, int] = pairs.iter().copied().collect()
    println(m[1], m[2], m.len())
    cs = ['h', 'e', 'y']
    word: String = cs.iter().copied().collect()
    println(word)
    parts = ["ab".to_string(), "cd".to_string()]
    joined: String = parts.iter().cloned().collect()
    println(joined)
    t: Tally = xs.iter().copied().collect()
    println(t.counts[3], t.counts[1], t.counts[2])
