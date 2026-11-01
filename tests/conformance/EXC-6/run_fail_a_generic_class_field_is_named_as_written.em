#$ test: run-fail
#$ rules: EXC-6, DIA-23
#$ profiles: debug, development
#$ panics: exclusivity violation: write access to Bag[i64].items while a read access to Bag[i64].items is active
# `[DIA-23]` — a run-time panic names Ember entities, never a C symbol: the
# field of a generic class instance is named by the class as written, with
# its arguments (`Bag[i64].items`), not by the instance's C name
# (`Bag_i64.items`; D-461).

[T]
class Bag:
    items: Array[T]

    fn init(x: T):
        this.items = [x]

    fn grow(x: T):
        this.items.push(owns x)

fn main():
    b = Bag[int](1)
    c = b
    v = b.items[..]
    c.grow(5)
    println(v)
