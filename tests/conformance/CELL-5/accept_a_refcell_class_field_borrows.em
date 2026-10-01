#$ test: run-pass
#$ rules: CELL-5, EXC-18, EXC-19
#$ profiles: debug, release
#$ stdout: [1, 2, 3]
#$ stdout: [1, 2, 3, 4]
#$ stdout: [1, 2, 3, 4] [1, 2, 3, 4]
# `[CELL-5]` — a `RefCell` in a class field borrows like any other: its own
# counter rules what is borrowed inside it, so `borrow` and `borrow_mut` read
# the class field that holds the cell (VIII.3: a method taking `self` on a
# field is a read), and their bookkeeping is no write to it (D-454). Two
# shared guards through two handles share the cell.

class Holder:
    cell: RefCell[Array[int]]

    fn init(mut self):
        self.cell = RefCell([1, 2, 3])

    fn add(self, x: int):
        with g = self.cell.borrow_mut():
            g.push(x)

fn main():
    b = Holder()
    with g = b.cell.borrow():
        println(g)
    b.add(4)
    with g = b.cell.borrow():
        println(g)
    c = b
    with g = b.cell.borrow():
        with h = c.cell.borrow():
            println(g, h)
