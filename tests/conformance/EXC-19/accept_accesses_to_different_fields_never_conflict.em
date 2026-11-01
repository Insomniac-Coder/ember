#$ test: run-pass
#$ rules: EXC-19, EXC-1, EXC-15
#$ profiles: debug, development, release
#$ stdout: [1, 2, 3] [1, 2, 3, 1, 2, 3]
# `[EXC-19]` — access state is per field: reading `items` while writing `log`,
# through one handle or two, and inside a `mut self` method, never conflicts.

class Bag:
    items: Array[int]
    log: Array[int]

    fn init():
        this.items = [1, 2, 3]
        this.log = []

    fn copy_items():
        for x in this.items:
            this.log.push(owns x)

fn main():
    b = Bag()
    c = b
    for x in b.items:
        c.log.push(owns x)
    b.copy_items()
    println(b.items, b.log)
