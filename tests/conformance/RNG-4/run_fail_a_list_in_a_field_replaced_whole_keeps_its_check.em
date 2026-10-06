#$ test: run-fail
#$ rules: RNG-4
#$ profiles: debug, release, shipping
#$ panics: integer overflow
# `[RNG-4]` (ADR-142) — a field given a whole list holds that list's elements:
# a call's result carries no fact.

class Bag:
    items: Array[i32]

    fn init(mut self):
        self.items = []

fn big_list(top: i32) -> Array[i32]:
    xs: Array[i32] = []
    xs.push(top)
    xs.push(10)
    return xs

fn main():
    b = Bag()
    for i in 0..10:
        b.items.push((i % 7) as i32)
    b.items = big_list(2147483642)
    c = Bag()
    c.items = big_list(0)
    total: i32 = 0
    for i in 0..2:
        total = total + b.items[i]
    println(total, len(c.items))
