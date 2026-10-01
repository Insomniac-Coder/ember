#$ test: run-pass
#$ rules: EXC-18, EXC-15, LT-1
#$ profiles: debug, release
#$ stdout: [1, 2, 3, 0]
#$ stdout: [7, 2, 3, 0]
# `[LT-1]` lets a `mut self` method return a view of its receiver, and
# `[EXC-18]` carries the field's access to the caller once the call ends. The
# call's own whole-object write (`[EXC-15]`) ends first, then the carried
# access begins, so a `mut self` getter can be called (D-451).

class Bag:
    items: Array[int]

    fn init(mut self):
        self.items = [1, 2, 3]

    fn grown_view(mut self) -> Span[int]:
        self.items.push(0)
        return self.items

    fn slot(mut self, i: int) -> ref mut int:
        return ref mut self.items[i]

fn main():
    b = Bag()
    v = b.grown_view()
    println(v)
    r = b.slot(0)
    r = 7
    println(b.items)
