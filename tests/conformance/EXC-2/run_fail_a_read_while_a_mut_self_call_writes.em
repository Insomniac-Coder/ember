#$ test: run-fail
#$ rules: EXC-2, EXC-15, EXC-19
#$ profiles: debug, development, release
#$ panics: exclusivity violation: read access to Bag.items while a write access to Bag.items is active
# `[EXC-2]`, `[EXC-15]` — a call of a method that changes its object (`count_other` pushes to
# `this.items`) writes every field of it until it returns; reading one of them through another
# handle to that object during the call panics.

class Bag:
    items: Array[int]

    fn init():
        this.items = [1, 2, 3]

    fn count_other(other: Bag) -> int:
        this.items.push(0)
        return len(other.items)

fn main():
    b = Bag()
    println(b.count_other(b))
