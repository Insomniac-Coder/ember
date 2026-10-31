#$ test: run-pass
#$ rules: LT-1a, LT-1, BRW-2
#$ stdout: ccc
#$ stdout: a ccc
#$ stdout: bb
#$ stdout: ccc
#$ stdout: bb
#$ stdout: a
# `[LT-1a]` (ODR-196) — `@borrows(self.owner)`: the result of `next` borrows what the iterator's
# `owner` field borrows, the words, not the iterator, and the call's loan on the iterator ends
# with the call. So an item is kept while `next` is called again, two items are held at once,
# and an item outlives the iterator it came from. In a plain type's method, a generic type's,
# and a generic extension's; and with a receiver taken `owned` or as a copy, whose field's borrow
# is the same.

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

@view
[T]
struct Each:
    owner: ref Array[T]
    at: int

    fn next() -> Option[ref T] from owner:
        if this.at >= this.owner.len():
            return None
        i = this.at
        this.at += 1
        return Some(ref this.owner[i])

@view
[T]
struct Back:
    owner: ref Array[T]
    at: int

extend[T] Back[T]:
    fn next() -> Option[ref T] from owner:
        if this.at <= 0:
            return None
        this.at -= 1
        return Some(ref this.owner[this.at])

@view
struct Holder:
    owner: ref Array[String]
    at: int

    fn take() owns this -> ref String from owner:
        return ref this.owner[this.at]

    fn peek() -> ref String from owner:
        return ref this.owner[this.at]

# The item outlives `it`, a local of this function: it borrows `words`.
fn first_long(words: ref Array[String]) -> Option[ref String]:
    it = WordIter(words, 0)
    while true:
        match it.next():
            Some(w):
                if w.len() > 1:
                    return Some(w)
            None:
                return None
    return None

fn main():
    words: Array[String] = ["a", "ccc", "bb"]
    best = ""
    it = WordIter(ref words, 0)
    while true:
        match it.next():
            Some(w):
                if w.len() > best.len():
                    best = w
            None:
                break
    println(best)
    e = Each(ref words, 0)
    a = e.next()
    b = e.next()
    match (a, b):
        (Some(x), Some(y)):
            println(x, y)
        _:
            pass
    k = Back(ref words, words.len())
    match k.next():
        Some(z):
            println(z)
        None:
            pass
    match first_long(words):
        Some(w):
            println(w)
        None:
            pass
    println(Holder(ref words, 2).take())
    println(Holder(ref words, 0).peek())
