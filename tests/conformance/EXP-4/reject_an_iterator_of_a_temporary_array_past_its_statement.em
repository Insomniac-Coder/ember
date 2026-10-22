#$ test: compile-fail
#$ rules: EXP-4, BRW-9
## `[EXP-4]` — outside a `for` header a temporary array ends with its
## statement, so an iterator or an element reference made from it may not be
## used after it (D-816).

fn make() -> Array[int]:
    return [3, 4]

fn main():
    it = make().iter() #$ error[E3060]: this temporary is dropped at the end of its statement while it is still borrowed
    println(it.count())
    first = make().first() #$ error[E3060]: this temporary is dropped at the end of its statement while it is still borrowed
    println(first)
