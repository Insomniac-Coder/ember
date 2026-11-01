#$ test: run-fail
#$ rules: RNG-4
#$ profiles: debug, development, release
#$ panics: integer overflow
# `[RNG-4]` (ADR-142) — a field's list lent mutably to a function may get any
# value there: its elements carry no fact, and the sum keeps its check.

class Bag:
    items: Array[i32]

    fn init():
        this.items = []

fn put(ref mut xs: Array[i32], v: i32):
    xs.push(owns v)

fn main():
    b = Bag()
    for i in 0..10:
        b.items.push(owns ((i % 7) as i32))
    put(ref mut b.items, 1073741823)
    put(ref mut b.items, 1073741823)
    total: i32 = 0
    for i in 0..12:
        total = total + b.items[i]
    println(total)
