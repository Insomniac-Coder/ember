#$ test: run-fail
#$ rules: EXC-15, DRP-1, EXC-1
#$ profiles: debug, release
#$ panics: exclusivity violation: write access to Bag.items while a write access to Bag.items is active
# `[DRP-1]` declares `fn drop(mut self)`, and `[EXC-15]` gives a `mut self`
# method a write access to every field from entry to return. The runtime is
# what calls `drop`, so it begins that write: a write to a field through
# another path while `drop` runs panics, where it would otherwise free what a
# view `drop` holds points into (D-453).

class Bag:
    items: Array[int]

    fn init(mut self):
        self.items = [1, 2, 3, 4, 5, 6, 7, 8]

    fn reset(self):
        self.items = [0]

    fn drop(mut self):
        v = self.items[..]
        self.reset()
        println(v)

fn main():
    b = Bag()
    println(len(b.items))
