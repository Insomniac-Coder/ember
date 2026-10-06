#$ test: run-fail
#$ rules: RNG-4
#$ profiles: debug, release, shipping
#$ panics: integer overflow
# `[RNG-4]` (ADR-142) — an element written by index in another function is a
# store into the field's lists like a `push`.

class Bag:
    items: Array[i32]

    fn init(mut self):
        self.items = []

fn set_first(mut b: Bag, v: i32):
    b.items[0] = v

fn main():
    b = Bag()
    for i in 0..10:
        b.items.push((i % 7) as i32)
    set_first(b, 2147483642)
    set_first(b, 2147483642)
    total: i32 = 0
    for i in 0..10:
        total = total + b.items[i]
    println(total)
