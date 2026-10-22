#$ test: run-pass
#$ rules: CTL-1, LT-1a, BRW-2
#$ stdout: bbb
#$ stdout: bbb
#$ stdout: yyyy
#$ stdout: 3
#$ stdout: bbb
#$ stdout: a
#$ stdout: three
#$ stdout: ccc
# `[CTL-1]` — a `for` over a place borrows the place and yields `ref T` into it. A `Map`'s keys
# (`for k in m:`, `m.keys()`), a `Set`'s elements, a map's values and items, and the parts of a
# split string borrow what they are read from, not the iterator (`@borrows(self.owner)` on the
# standard library's `next`, `[LT-1a]`, ODR-196): the "best so far" idiom keeps one across turns
# and after the loop, and an iterator's item outlives the iterator. Also an `Iterator` value of
# the program's own, declared the same way.

@view
struct WordIter:
    owner: ref Array[String]
    at: int

extend WordIter implements Iterator:
    type Item = ref String

    @borrows(self.owner)
    fn next(mut self) -> Option[ref String]:
        if self.at >= self.owner.len():
            return None
        i = self.at
        self.at += 1
        return Some(ref self.owner[i])

fn longest_key(m: ref Map[String, int]) -> str:
    best = ""
    for k in m.keys():
        if k.len() > best.len():
            best = k
    return best

fn first_key(m: ref Map[String, int]) -> Option[ref String]:
    it = m.keys()
    return it.next()

fn main():
    m: Map[String, int] = Map()
    m.insert("a", 1)
    m.insert("bbb", 2)
    m.insert("cc", 3)
    best = ""
    for k in m:
        if k.len() > best.len():
            best = k
    println(best)
    println(longest_key(m))
    s: Set[String] = Set()
    s.add("x")
    s.add("yyyy")
    top = ""
    for v in s:
        if v.len() > top.len():
            top = v
    println(top)
    most = 0
    for v in m.values():
        if v > most:
            most = v
    println(most)
    pair_key = ""
    for (k, v) in m.items():
        if v == 2:
            pair_key = k
    println(pair_key)
    match first_key(m):
        Some(k):
            println(k)
        None:
            pass
    longest = ""
    for w in "one two three".split(" "):
        if w.len() > longest.len():
            longest = w
    println(longest)
    words: Array[String] = ["a", "ccc", "bb"]
    widest = ""
    for w in WordIter(ref words, 0):
        if w.len() > widest.len():
            widest = w
    println(widest)
