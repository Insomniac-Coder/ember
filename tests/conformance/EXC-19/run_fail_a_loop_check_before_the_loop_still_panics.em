#$ test: run-fail
#$ rules: EXC-8, EXC-19, PHIL-5
#$ profiles: debug, release, shipping
#$ stdout: before
#$ panics: exclusivity violation: write access to Bag.items while a read access to Bag.items is active
# `[EXC-8]` — the loop's one check, run before it, fails as the first turn's
# would: `bags[0].items` is viewed while the loop pushes to it. What the
# program printed before the loop is out; nothing after it is.

class Bag:
    items: Array[int]

    fn init(mut self):
        self.items = []

fn main():
    bags: Array[Bag] = []
    for k in 0..2:
        b = Bag()
        b.items.push(7)
        bags.push(b)
    view = bags[0].items.as_span()
    println("before")
    sink = bags[0]
    for i in 0..10:
        sink.items.push(i)
    println("after", view[0])
