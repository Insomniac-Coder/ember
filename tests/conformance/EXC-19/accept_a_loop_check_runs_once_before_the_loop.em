#$ test: run-pass
#$ rules: EXC-8, EXC-19, PHIL-5
#$ profiles: debug, release, shipping
#$ stdout: 0 3000 7
# `[EXC-8]` — each `push` checks that no access to `sink.items` is active.
# The loop changes no access and runs no Ember code, and `sink` stays the
# same object, so the check runs once, before the loop, and only when the
# loop runs: `bags[0].items` is viewed, and pushing to it for zero turns
# checks nothing, so it does not panic; pushing to `bags[1].items` 3000 times
# checks it once.

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
    turns = 0
    for round in 0..2:
        sink = bags[round]
        for i in 0..turns:
            sink.items.push(i)
        turns = 3000
    println(len(bags[0].items) - 1, len(bags[1].items) - 1, view[0])
