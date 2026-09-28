#$ test: run-fail
#$ rules: EXC-18, EXC-1, EXC-19
#$ profiles: debug, release, shipping
#$ panics: exclusivity violation: write access to Bag.items while a read access to Bag.items is active
# `[EXC-18]` — a field passed straight to a borrowed parameter is read for the
# whole call (F1).

class Bag:
    items: Array[int]

    fn init(mut self):
        self.items = [1, 2, 3]

fn use_view(v: Span[int], other: Bag) -> int:
    alias = other
    alias.items.push(99)
    return v[0]

fn main():
    b = Bag()
    c = b
    println(use_view(b.items, c))
