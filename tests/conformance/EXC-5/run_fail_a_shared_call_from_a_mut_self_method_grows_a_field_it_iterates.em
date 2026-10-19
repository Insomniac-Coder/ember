#$ test: run-fail
#$ rules: EXC-5, EXC-2, EXC-15
#$ profiles: debug, development
#$ panics: exclusivity violation: write access to Bag.items while a read access to Bag.items is active
# `[EXC-5]` — what a `mut self` method lends to a shared call on `self` is a read of the whole
# object, not its write: `walk` iterates `self.items` (its own access, covered by its write), and
# `grow`, a shared method, pushes to `self.items`, which would move the buffer the loop reads.
# The push conflicts with the lent read and panics (`[EXC-2]`).
class Bag:
    items: Array[int]

    fn init(mut self):
        self.items = [1]

    fn grow(self):
        self.items.push(9)

    fn walk(mut self):
        for x in self.items:
            println(x)
            self.grow()

fn main():
    b = Bag()
    b.walk()
