#$ test: run-pass
#$ rules: EXC-2, EXC-19
#$ profiles: debug, release, shipping
#$ stdout: [1, 2] [2, 3] 3 3
# `[EXC-2]` — reads do not conflict with each other: two views of one field and
# a read of it through another handle are all live at once.

class Bag:
    items: Array[int]
    log: Array[int]

    fn init(mut self):
        self.items = [1, 2, 3]
        self.log = []

    fn bump(mut self):
        self.items.push(len(self.items) + 1)

fn main():
    b = Bag()
    c = b
    first = b.items[..2]
    second = c.items[1..]
    println(first, second, len(c.items), b.items[2])
