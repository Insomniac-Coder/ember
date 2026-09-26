#$ test: run-fail
#$ rules: EXC-19, EXC-15, EXC-1
#$ profiles: debug, release, shipping
#$ panics: exclusivity violation: write access to Bag.items while a read access to Bag.items is active
# `[EXC-19]` — a `mut self` call checks every field word of its object at entry;
# a live view of one of them is a conflict, named by its field.

class Bag:
    items: Array[int]
    log: Array[int]

    fn init(mut self):
        self.items = [1, 2, 3]
        self.log = []

    fn bump(mut self):
        self.items.push(len(self.items) + 1)

fn main():
    b = Bag()
    c = b
    v = b.items[..2]
    c.bump()
    println(v)
