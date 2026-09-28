#$ test: run-fail
#$ rules: OPT-2
#$ profiles: debug, release, shipping
#$ panics: index 1 is out of bounds for a length of 0
# `empty` clears `bag.items` through its handle, which its summary shows, so
# the loop keeps every check and panics on the second pass, as it always did.

class Bag:
    items: Array[int]

    fn init(mut self):
        self.items = []

fn empty(mut bag: Bag):
    bag.items.clear()

fn main():
    bag = Bag()
    for i in 0..4:
        bag.items.push(i)
    total = 0
    for i in 0..4:
        total = total + bag.items[i]
        empty(bag)
    println(total)
