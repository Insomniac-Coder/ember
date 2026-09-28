#$ test: run-pass
#$ rules: OPT-2
#$ profiles: debug, release, shipping
#$ stdout: 12
#$ assert-c-count: contains("ember_panic_bounds(") == 1
#$ assert-c-count: contains("ember_ck_add_i64(") == 2
# `double` writes nothing the loop reads, which its summary shows, so the
# loop over `bag.items` is versioned: the add appears in both copies and the
# bounds check only in the checked one.

class Bag:
    items: Array[int]

    fn init(mut self):
        self.items = []

fn double(x: int) -> int:
    return x * 2

fn main():
    bag = Bag()
    for i in 0..4:
        bag.items.push(i)
    total = 0
    for i in 0..4:
        total = total + double(bag.items[i])
    println(total)
