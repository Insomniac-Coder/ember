#$ test: run-pass
#$ rules: RNG-4, TYP-8
#$ profiles: debug, release, shipping
#$ stdout: 8991000
#$ assert-c-count: contains("ember_ck_") == 0
# `[RNG-4]` (ADR-142) — the lists in a field change only through the whole
# program's stores into them. Every store into `Bag.items` is a `push` of
# `i % 7`, so its elements are 0 to 6; the sum over 1,000 of them on each of
# 3,000 turns is at most 18,000,000, and `total + b.items[i]` keeps no check
# (D-530: the inner loop's 1,000 runs are counted on every outer turn).

class Bag:
    items: Array[int]

    fn init(mut self):
        self.items = []

fn main():
    bags: Array[Bag] = []
    for k in 0..2:
        b = Bag()
        for i in 0..1000:
            b.items.push(i % 7)
        bags.push(b)
    total = 0
    for round in 0..3000:
        b = bags[round % 2]
        for i in 0..1000:
            total = total + b.items[i]
    println(total)
