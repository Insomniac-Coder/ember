#$ test: run-pass
#$ rules: CELL-6, EXC-18
#$ profiles: debug, release
#$ stdout: true
#$ stdout: [1, 2, 3, 4]
# `[CELL-6]` — `try_borrow` of a `RefCell` held in a class field returns
# `None` while a conflicting guard lives and a guard when free: its guard is
# made on the success path only, and holds the field's access on that path
# (D-455, D-454).

class Holder:
    cell: RefCell[Array[int]]

    fn init(mut self):
        self.cell = RefCell([1, 2, 3])

fn main():
    b = Holder()
    c = b
    with g = b.cell.borrow_mut():
        g.push(4)
        r = c.cell.try_borrow()
        println(r.is_none())
    match c.cell.try_borrow():
        Some(guard):
            println(guard)
        None:
            println("busy")
