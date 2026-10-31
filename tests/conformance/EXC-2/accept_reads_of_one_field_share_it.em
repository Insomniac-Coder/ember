#$ test: run-pass
#$ rules: EXC-2, EXC-19
#$ profiles: debug, development, release
#$ stdout: [1, 2] [2, 3] 3 3
# `[EXC-2]` — reads do not conflict with each other: two views of one field and
# a read of it through another handle are all live at once.

class Bag:
    items: Array[int]
    log: Array[int]

    fn init():
        this.items = [1, 2, 3]
        this.log = []

    fn bump():
        this.items.push(len(this.items) + 1)

fn main():
    b = Bag()
    c = b
    first = b.items[..2]
    second = c.items[1..]
    println(first, second, len(c.items), b.items[2])
