#$ test: compile-fail
#$ rules: BRW-17
#$ help: println(bag.items[0])
## design rules: R8, R7
## R8 (b): a kept view is an address, never a position; a push that may move the list it points into
## ends it (E3060 at a later use).

struct Bag:
    items: Array[int]

struct Head:
    view: Span[int]

fn main():
    bag = Bag(items=[1, 2])
    h = Head(view=bag.items.as_span())
    bag.items.push(3)
    println(h.view[0])    #$ error[E3060]: `bag.items.push(3)` (line 16) may move the list that `h.view` points into (line 15), and `h.view` is used on line 17; use `h.view` before line 16, or read the number again after it: write `println(bag.items[0])` in place of line 17
