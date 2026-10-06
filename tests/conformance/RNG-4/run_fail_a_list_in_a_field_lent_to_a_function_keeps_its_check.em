#$ test: run-fail
#$ rules: RNG-4
#$ profiles: debug, release, shipping
#$ panics: integer overflow
# `[RNG-4]` (ADR-142) — a field's list lent mutably to a function may get any
# value there: its elements carry no fact, and the sum keeps its check.

class Bag:
    items: Array[i32]

    fn init(mut self):
        self.items = []

fn put(mut xs: Array[i32], v: i32):
    xs.push(v)

fn main():
    b = Bag()
    for i in 0..10:
        b.items.push((i % 7) as i32)
    put(b.items, 1073741823)
    put(b.items, 1073741823)
    total: i32 = 0
    for i in 0..12:
        total = total + b.items[i]
    println(total)
