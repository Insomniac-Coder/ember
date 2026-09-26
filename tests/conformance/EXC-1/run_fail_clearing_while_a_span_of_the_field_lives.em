#$ test: run-fail
#$ rules: EXC-1, EXC-18, EXC-19
#$ profiles: debug, release, shipping
#$ panics: exclusivity violation: write access to Bag.items while a read access to Bag.items is active
# `[EXC-1]` (D-202) — a `Span` of a class field holds the field's read access as a
# slice does.

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
    v: Span[int] = b.items
    c.items.clear()
    println(v)
