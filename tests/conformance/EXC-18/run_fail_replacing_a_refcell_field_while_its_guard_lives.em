#$ test: run-fail
#$ rules: EXC-18, EXC-1, CELL-5
#$ profiles: debug, release
#$ panics: exclusivity violation: write access to Holder.cell while a read access to Holder.cell is active
# `[EXC-18]` — a guard from a `RefCell` class field holds the field's read
# access while it lives: replacing the whole cell through another handle
# meanwhile panics, where it would free the value the guard points into.
# Reaching inside the cell is a read (D-454); replacing it is a write.

class Holder:
    cell: RefCell[Array[int]]

    fn init(mut self):
        self.cell = RefCell([1, 2, 3])

    fn reset(self):
        self.cell = RefCell([9])

fn main():
    b = Holder()
    c = b
    with g = b.cell.borrow():
        c.reset()    #$ warning[L3011]: live across this call
        println(g)
