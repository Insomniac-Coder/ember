#$ test: run-pass
#$ rules: EXC-15, DRP-1, CLS-6
#$ profiles: debug, release
#$ stdout: 4
#$ stdout: bag [1, 2, 3, 4, 4] [1, 8]
#$ stdout: base [1, 8, 9]
# `[EXC-15]` — while `drop` runs, the object's write access is held (D-453);
# `drop` calling `mut self` methods on itself, viewing its fields, and the base
# class's `drop` running after it (`[CLS-6]`) are all covered by it.

open class Base:
    log: Array[int]

    fn init(mut self):
        self.log = [1]

    fn note(mut self, x: int):
        self.log.push(x)

    fn drop(mut self):
        self.note(9)
        println(f"base {self.log}")

class Bag(Base):
    items: Array[int]

    fn init(mut self):
        super.init()
        self.items = [1, 2, 3]

    fn grow(mut self):
        self.items.push(4)

    fn drop(mut self):
        self.grow()
        self.note(8)
        v = self.items[..]
        println(f"bag {v} {self.log}")

fn main():
    b = Bag()
    b.grow()
    println(len(b.items))
