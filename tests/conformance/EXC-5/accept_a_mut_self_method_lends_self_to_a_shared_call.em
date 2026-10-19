#$ test: run-pass
#$ rules: EXC-5, EXC-15, EXC-19
#$ profiles: debug, development, release
#$ stdout: [1, 2] true 3 2
# `[EXC-5]` — "Accesses nested inside the same `mut self` method are reborrows": a `mut self`
# method may call a shared method on `self` (`self.has(x)`) or give `self` to a function taking it
# borrowed (`size(self)`), and their reads of the object's fields are its own. The method's
# whole-object write (`[EXC-15]`) is lent to the call as a read of the whole object and taken
# back after it (D-690: the callee's reads checked against the write, and panicked).
class Bag:
    items: Array[int]
    count: int

    fn init(mut self):
        self.items = []
        self.count = 0

    fn has(self, x: int) -> bool:
        return self.items.contains(x)

    fn total(self) -> int:
        return self.count + self.items.len()

    fn add(mut self, x: int):
        if not self.has(x):
            self.items.push(x)
            self.count += size(self) - self.items.len() + 1

fn size(b: Bag) -> int:
    return b.items.len()

fn main():
    b = Bag()
    b.add(1)
    b.add(2)
    b.add(1)
    println(b.items, b.has(2), b.total() - 1, b.count)
