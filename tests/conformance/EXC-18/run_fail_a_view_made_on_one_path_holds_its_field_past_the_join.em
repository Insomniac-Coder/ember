#$ test: run-fail
#$ rules: EXC-18, EXC-1
#$ profiles: debug, release
#$ panics: exclusivity violation: write access to Bag.items while a read access to Bag.items is active
# `[EXC-18]` — when the branch that made a view of a class field ran, the view
# holds the field's read access past the join until its last use: a write
# through another handle meanwhile panics (D-455 keeps the access only on the
# path that began it, not less).

class Bag:
    items: Array[int]

    fn init(mut self):
        self.items = [1, 2, 3]

    fn grow(self):
        self.items.push(9)

fn main():
    b = Bag()
    c = b
    other: Array[int] = [5]
    v: Span[int] = other[..]
    if len(other) == 1:
        v = b.items[..]
    c.grow()
    println(v)
