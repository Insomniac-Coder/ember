#$ test: run-fail
#$ rules: OPT-2
#$ profiles: debug, development, release
#$ panics: index 1 is out of bounds for a length of 0
# Dropping `e` at the end of each pass runs `Emptier.drop`, which clears
# `bag.items`: the loop keeps every check and panics on the second pass.

class Bag:
    items: Array[int]

    fn init():
        this.items = []

class Emptier:
    target: Bag

    fn drop():
        this.target.items.clear()

fn main():
    bag = Bag()
    for i in 0..4:
        bag.items.push(owns i)
    total = 0
    for i in 0..4:
        e = Emptier(bag)
        total = total + bag.items[i]
    println(total)
