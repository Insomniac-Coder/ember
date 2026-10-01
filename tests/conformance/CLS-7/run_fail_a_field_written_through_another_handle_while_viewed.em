#$ test: run-fail
#$ rules: CLS-7, EXC-16, EXC-1
#$ profiles: debug, release
#$ panics: exclusivity violation: write access to Bag.items while a read access to Bag.items is active
# `[EXC-16]` — assigning a class field through another handle is a write
# access to it: while a view of the field lives, it panics, where it would
# free the array the view points into (D-462).

class Bag:
    items: Array[int]

    fn init(mut self):
        self.items = [1, 2, 3]

fn main():
    b = Bag()
    c = b
    v = b.items[..]
    c.items = [9]
    println(v)
