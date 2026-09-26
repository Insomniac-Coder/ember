#$ test: run-pass
#$ rules: EXC-19, EXC-1, EXC-15
#$ profiles: debug, release, shipping
#$ stdout: [1, 2, 3] [1, 2, 3, 1, 2, 3]
# `[EXC-19]` — access state is per field: reading `items` while writing `log`,
# through one handle or two, and inside a `mut self` method, never conflicts.

class Bag:
    items: Array[int]
    log: Array[int]

    fn init(mut self):
        self.items = [1, 2, 3]
        self.log = []

    fn copy_items(mut self):
        for x in self.items:
            self.log.push(x)

fn main():
    b = Bag()
    c = b
    for x in b.items:
        c.log.push(x)
    b.copy_items()
    println(b.items, b.log)
