#$ test: run-fail
#$ rules: EXC-1, EXC-18, EXC-19
#$ profiles: debug, release, shipping
#$ panics: exclusivity violation: write access to Bag.items while a read access to Bag.items is active
# `[EXC-1]` (D-202) — a view of a class field is a long-term read access to the
# field until its last use. A push through an aliasing handle would reallocate
# the buffer the view points into; it panics instead of using freed memory.

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
    v = b.items[1..]
    c.items.push(4)
    println(v)
