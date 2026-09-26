#$ test: run-fail
#$ rules: EXC-2, EXC-15, EXC-19
#$ profiles: debug, release, shipping
#$ panics: exclusivity violation: read access to Bag.items while a write access to Bag.items is active
# `[EXC-2]`, `[EXC-15]` — a `mut self` call writes every field of its object
# until it returns; reading one of them through another handle to that object
# during the call panics.

class Bag:
    items: Array[int]

    fn init(mut self):
        self.items = [1, 2, 3]

    fn count_other(mut self, other: Bag) -> int:
        return len(other.items)

fn main():
    b = Bag()
    println(b.count_other(b))
