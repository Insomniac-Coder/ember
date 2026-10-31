#$ test: run-pass
#$ rules: LT-1a, GRM-41, FN-3
#$ stdout: ccc
#$ stdout: bb
#$ stdout: abc
## `[LT-1a]`, `[GRM-41]` — `-> T from name` after the result type says what a returned view points
## into: a parameter, or a field of `this` written bare. The result of `next` points into what
## `owner` views, not the iterator, so an item outlives the iterator it came from; the result of
## `pick` points into `a` alone, so it outlives `b`.

@view
struct WordIter:
    owner: ref Array[String]
    at: int

    fn next() -> Option[ref String] from owner:
        if this.at >= this.owner.len():
            return None
        i = this.at
        this.at += 1
        return Some(ref this.owner[i])

fn longest(words: ref Array[String]) -> Option[ref String] from words:
    it = WordIter(words, 0)
    best = 0
    at = 0
    while it.next() exists as w:
        if w.len() > words[best].len():
            best = at
        at += 1
    if words.is_empty():
        return None
    return Some(ref words[best])

fn pick(a: StrView, b: StrView) -> StrView from a:
    if b.len() > 100:
        return a
    return a

fn main():
    words: Array[String] = ["a", "ccc", "bb"]
    if longest(words) exists as w:
        println(w)
    it = WordIter(ref words, 2)
    if it.next() exists as w:
        println(w)
    kept = String("abc")
    held: StrView = ""
    if true:
        other = String("x")
        held = pick(kept.as_str(), other.as_str())
    println(held)
