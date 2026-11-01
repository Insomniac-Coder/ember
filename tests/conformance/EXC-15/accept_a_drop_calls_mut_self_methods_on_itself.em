#$ test: run-pass
#$ rules: EXC-15, DRP-1, CLS-6
#$ profiles: debug, development
#$ stdout: 4
#$ stdout: bag [1, 2, 3, 4, 4] [1, 8]
#$ stdout: base [1, 8, 9]
# `[EXC-15]` — while `drop` runs, the object's write access is held (D-453);
# `drop` calling `mut self` methods on itself, viewing its fields, and the base
# class's `drop` running after it (`[CLS-6]`) are all covered by it.

open class Base:
    log: Array[int]

    fn init():
        this.log = [1]

    fn note(x: int):
        this.log.push(owns x)

    fn drop():
        this.note(9)
        println(f"base {this.log}")

class Bag(Base):
    items: Array[int]

    fn init():
        this.items = [1, 2, 3]
        super.init()

    fn grow():
        this.items.push(owns 4)

    fn drop():
        this.grow()
        this.note(8)
        v = this.items[..]
        println(f"bag {v} {this.log}")

fn main():
    b = Bag()
    b.grow()
    println(len(b.items))
