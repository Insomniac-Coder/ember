#$ test: run-fail
#$ rules: EXC-1, EXC-19
#$ profiles: debug, release, shipping
#$ panics: exclusivity violation: write access to Bag.items while a read access to Bag.items is active
# `[EXC-1]` — iterating a field reads it for the whole loop; growing it through
# an aliasing handle inside the loop panics.

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
    for x in b.items:
        c.items.push(x)
    println(b.items)
