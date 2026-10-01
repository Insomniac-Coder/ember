#$ test: run-fail
#$ rules: CELL-5
#$ profiles: debug, release
#$ panics: RefCell already mutably borrowed (borrowed at
# `[CELL-5]` — in a class field, a `RefCell`'s own counter still refuses a
# `borrow_mut` while a shared guard lives, through another handle too: the
# panic is the cell's, naming where the conflicting borrow began, not an
# exclusivity violation on the field (D-454).

class Holder:
    cell: RefCell[Array[int]]

    fn init(mut self):
        self.cell = RefCell([1, 2, 3])

fn main():
    b = Holder()
    c = b
    with g = b.cell.borrow():
        with h = c.cell.borrow_mut():
            h.push(4)
        println(g)
