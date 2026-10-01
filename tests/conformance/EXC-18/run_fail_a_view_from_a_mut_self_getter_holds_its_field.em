#$ test: run-fail
#$ rules: EXC-18, EXC-1
#$ profiles: debug, release
#$ panics: exclusivity violation: write access to Bag.items while a read access to Bag.items is active
# `[EXC-18]` — a view a `mut self` method returns holds its field's read access
# from the end of the call to the view's last use: a write through another
# handle meanwhile panics (D-451 moved where the access begins, not how long
# it lasts).

class Bag:
    items: Array[int]

    fn init(mut self):
        self.items = [1, 2, 3]

    fn grown_view(mut self) -> Span[int]:
        self.items.push(0)
        return self.items

fn main():
    b = Bag()
    c = b
    v = b.grown_view()
    c.items.push(5)
    println(v)
