#$ test: compile-fail
#$ rules: EXP-4, BRW-9, TYP-38
# `[EXP-4]` (D-342) — a temporary ends with its statement whether or not it
# has a destructor, so a view of one may not be used after it: `W3(3)` needs
# no drop, and `inner` returns a reference into it; a list literal in a
# `Span` position is a view of a temporary array (`[TYP-38]`). Both compiled
# before, and read a C local the program had stopped owning.

struct W3:
    item: i64

struct Window:
    items: Span[int]

fn inner(w: W3) -> ref i64:
    return ref w.item

fn main():
    r = inner(W3(3)) #$ error[E3060]: this temporary is dropped at the end of its statement while it is still borrowed
    println(r)
    w = Window(items = [1, 2, 3]) #$ error[E3060]: this temporary is dropped at the end of its statement while it is still borrowed
    println(w.items[0])
