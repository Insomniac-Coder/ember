#$ test: run-pass
#$ rules: EXC-1, EXC-18, EXC-19
#$ profiles: debug, release, shipping
#$ stdout: [2, 3]
#$ stdout: [1, 2, 3, 4]
# `[EXC-18]` — a view's access to its field ends at the view's last use, so a
# write through another handle after it is no conflict.

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
    v = b.items[1..]
    println(v)
    c.items.push(4)
    println(b.items)
