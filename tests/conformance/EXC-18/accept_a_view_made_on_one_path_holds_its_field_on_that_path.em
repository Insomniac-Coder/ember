#$ test: run-pass
#$ rules: EXC-18, EXC-2, EXC-1
#$ profiles: debug, release
#$ stdout: [5]
#$ stdout: [1, 2, 3, 9]
#$ stdout: [2, 3, 9]
#$ stdout: Some([7])
#$ stdout: [5]
#$ stdout: [1, 2, 3, 9, 9]
#$ stdout: [5]
#$ stdout: [1, 2, 3, 9, 9, 9, 9]
# `[EXC-18]` — a view of a class field holds the field's access while the loan
# lives on the path taken. A view made in one branch and kept past the join
# holds it only where that branch ran: the write through `c` meets no access
# when the view is of `other`. Choosing between two views of fields, or of one
# field, an `Option` of a view made on one path, and a view made on one turn
# of a loop all compile and run (D-455).

class Bag:
    items: Array[int]
    log: Array[int]

    fn init(mut self):
        self.items = [1, 2, 3]
        self.log = [7]

    fn grow(self):
        self.items.push(9)

fn main():
    b = Bag()
    c = b
    other: Array[int] = [5]
    v: Span[int] = other[..]
    if len(other) == 2:
        v = b.items[..]
    c.grow()
    println(v)
    w = b.items[..] if len(b.log) == 1 else b.log[..]
    println(w)
    x = b.items[..] if len(b.log) == 2 else b.items[1..]
    println(x)
    o: Option[Span[int]] = None
    if len(b.log) == 1:
        o = Some(b.log[..])
    println(o)
    for i in 0..3:
        s: Span[int] = other[..]
        if i == 1:
            s = b.items[..]
        println(s)
        c.grow()
    println(b.items)
