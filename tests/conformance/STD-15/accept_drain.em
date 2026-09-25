#$ test: run-pass
#$ rules: STD-15, CTL-3
#$ stdout:
#$ [2, 3] [0, 1, 4, 5, 6, 7]
#$ [6, 7] [0, 1, 4, 5]
#$ [0] [1, 4, 5]
#$ [1, 4] [5]
#$ [] [5]
#$ ['b', 'c', 'z'] ['a', 'd', 'e']
#$ [2, 3] [1]
# `[STD-15]` (ODR-031) — `drain(r)` moves the elements in `r` out, in order,
# as a new `Array`, and the rest close up. `r` is any range of integers:
# `a..b`, `a..` and `..b` (`[CTL-3]`), `a..=b`, and an empty one. The drained
# elements are moved, not copied: both arrays go on owning theirs.

fn main():
    xs = [0, 1, 2, 3, 4, 5, 6, 7]
    taken = xs.drain(2..4)
    println(taken, xs)
    tail = xs.drain(4..)
    println(tail, xs)
    head = xs.drain(..1)
    println(head, xs)
    mid = xs.drain(0..=1)
    println(mid, xs)
    none = xs.drain(1..1)
    println(none, xs)

    words = [String.from("a"), String.from("b"), String.from("c"), String.from("d")]
    out = words.drain(1..3)
    out.push(String.from("z"))
    words.push(String.from("e"))
    println(out, words)

    small: Array[u16] = [1, 2, 3]
    start: u16 = 1
    println(small.drain(start..3), small)
