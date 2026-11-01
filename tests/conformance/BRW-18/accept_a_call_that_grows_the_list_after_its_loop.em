#$ test: run-pass
#$ rules: BRW-18, EFF-1
#$ stdout: 1
#$ stdout: [1, 9]
# `[BRW-18]` — the rule holds only while the loop walks the list: `this.grow()` after the loop
# ends changes nothing a loop reads.

class Bag:
    items: Array[int]

    fn init():
        this.items = [1]

    fn grow():
        this.items.push(9)

    fn walk():
        for x in this.items:
            println(x)
        this.grow()

fn main():
    b = Bag()
    b.walk()
    println(b.items)
