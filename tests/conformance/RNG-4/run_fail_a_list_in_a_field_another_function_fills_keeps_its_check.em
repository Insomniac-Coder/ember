#$ test: run-fail
#$ rules: RNG-4
#$ profiles: debug, release, shipping
#$ panics: integer overflow
# `[RNG-4]` (ADR-142) — every function's stores into a field's lists count:
# `top_up` pushes a value near `i32`'s top, so the sum keeps its check.

class Bag:
    items: Array[i32]

    fn init(mut self):
        self.items = []

fn top_up(mut b: Bag, v: i32):
    b.items.push(v)

fn main():
    b = Bag()
    for i in 0..10:
        b.items.push((i % 7) as i32)
    top_up(b, 2147483642)
    top_up(b, 0)
    total: i32 = 0
    for i in 0..12:
        total = total + b.items[i]
    println(total)
