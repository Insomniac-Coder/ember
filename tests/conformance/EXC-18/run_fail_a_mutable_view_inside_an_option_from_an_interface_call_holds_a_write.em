#$ test: run-fail
#$ rules: EXC-18, EXC-2
#$ profiles: debug, development
#$ panics: exclusivity violation: read access to Bag.items while a write access to Bag.items is active
# `[EXC-18]` — a `ref mut` of a field carried out of an interface call is a
# write access until its last use, wherever it sits in the result: inside an
# `Option` too. A read through another handle meanwhile panics; as a read, it
# would have let a view begin that the write through the `ref mut` then
# freed (D-452).

interface Slots:
    fn slot(i: int) -> Option[ref mut int]

class Bag implements Slots:
    items: Array[int]

    fn init():
        this.items = [1, 2, 3]

    fn slot(i: int) -> Option[ref mut int]:
        return Some(ref mut this.items[i])

fn main():
    b = Bag()
    c = b
    h: Slots = b
    r = h.slot(0)
    println(len(c.items))
    match owns r:
        Some(x):
            x = 5
        None:
            pass
    println(b.items)
