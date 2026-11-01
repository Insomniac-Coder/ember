#$ test: run-fail
#$ rules: RNG-4
#$ profiles: debug, development, release
#$ panics: integer overflow
# `[RNG-4]` (ADR-142) — a field given a whole list holds that list's elements:
# a call's result carries no fact.

class Bag:
    items: Array[i32]

    fn init():
        this.items = []

fn big_list(top: i32) -> Array[i32]:
    xs: Array[i32] = []
    xs.push(owns top)
    xs.push(owns 10)
    return xs

fn main():
    b = Bag()
    for i in 0..10:
        b.items.push(owns ((i % 7) as i32))
    b.items = big_list(2147483642)
    c = Bag()
    c.items = big_list(0)
    total: i32 = 0
    for i in 0..2:
        total = total + b.items[i]
    println(total, len(c.items))
