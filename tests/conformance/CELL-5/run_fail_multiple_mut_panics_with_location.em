#$ test: run-fail
#$ rules: CELL-5
# `[CELL-5]` — two mutable borrows cannot be live at once. The first
# `borrow_mut` is active across the second, so the counter refuses with the
# first borrow's location.

fn main():
    c: RefCell[i32] = RefCell(1)
    with _a = c.borrow_mut():
        with _b = c.borrow_mut():
            println(1)
#$ panics: RefCell already mutably borrowed (borrowed at
