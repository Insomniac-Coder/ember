#$ test: run-fail
#$ rules: EXC-18, EXC-1, EXC-19
#$ profiles: debug, release, shipping
#$ panics: exclusivity violation: write access to Bag.items while a read access to Bag.items is active
# `[EXC-18]` — a view of a field whose last use is a call holds the field for the
# whole call, not just until the call begins. Here the callee frees the viewed
# buffer through another handle; that must panic, not read freed memory (F1).

class Bag:
    items: Array[int]

    fn init(mut self):
        self.items = [1, 2, 3]

    fn replace(self):
        self.items = [7]

fn show(v: Span[int], other: Bag):
    other.replace()
    println(v[0])

fn main():
    b = Bag()
    c = b
    show(b.items.as_span(), c)
