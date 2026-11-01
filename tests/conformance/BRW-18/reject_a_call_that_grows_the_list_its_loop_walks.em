#$ test: compile-fail
#$ rules: BRW-18, EFF-1
# `[BRW-18]` — while a loop walks a list, nothing changes it, through its own name or through any
# call that may change it (`[EFF-1]`): `walk`'s loop walks `this.items` and `this.grow()` pushes to
# it, which may move the list the loop reads. 0.9.10 stopped this program while it ran (`EXC-5`);
# 0.9.15 refuses it when compiling, at the change.

class Bag:
    items: Array[int]

    fn init():
        this.items = [1]

    fn grow():
        this.items.push(9)

    fn walk():
        for x in this.items:
            println(x)
            this.grow()    #$ error[E3020]

fn main():
    b = Bag()
    b.walk()
