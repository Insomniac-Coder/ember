# Clause LT-47.2: "After a call, a caller's place given to a `ref mut` parameter whose value is a view
# may point into what it pointed into before, or into any of the call's other arguments"; "A use of
# such a view after one of the call's arguments changed is `E3060`."
#$ test: compile-fail
#$ rules: LT-47, LT-1, BCK-1
# The spec's message, with this file's line numbers.

struct Holder:
    view: Span[int]

struct Bag:
    items: Array[int]

fn pick(ref mut v: Span[int], other: Span[int]):
    if other.len() > v.len():
        v = other

fn main():
    first = [1]
    h = Holder(view=first)
    bag = Bag([1, 2, 3])
    k = 4
    pick(ref mut h.view, bag.items)
    println(h.view.len())
    bag.items.push(owns k)
    println(h.view.len())    #$ error[E3060]: `bag.items.push(owns k)` (line 25) may move the list that `h.view` points into (`pick` may have put a view of `bag` there on line 23), and `h.view` is used on line 26; use `h.view` before line 25
