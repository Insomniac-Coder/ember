#$ test: run-fail
#$ rules: RNG-4, EXC-15
#$ profiles: debug, release, shipping
#$ panics: index 2 is out of bounds for a length of 0
# `[RNG-4]` — a list held by an object can change through any handle to it,
# so its length carries no fact: `k < bag.items.len()` removes nothing.

class Bag:
    items: Array[int]

    fn init(mut self):
        self.items = [1, 2, 3]

fn empty(mut b: Bag):
    b.items.clear()

fn main():
    bag = Bag()
    k = 2
    if k < bag.items.len():
        empty(bag)
        println(bag.items[k])
