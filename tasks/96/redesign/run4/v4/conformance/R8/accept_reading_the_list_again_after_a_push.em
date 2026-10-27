#$ test: run-pass
#$ rules: R8, R7
#$ stdout: 1 1
## R8 (b): the ways out E3060's message gives: the kept view is used before the push, and the number is
## read again through the list after it.

struct Bag:
    items: Array[int]

struct Head:
    view: Span[int]

fn main():
    bag = Bag(items=[1, 2])
    h = Head(view=bag.items.as_span())
    first = h.view[0]
    bag.items.push(3)
    println(first, bag.items[0])
