#$ test: run-fail
#$ rules: CELL-5
# `[CELL-5]` — `borrow` fails while a mutable borrow is active (the symmetric
# contention to the shared-then-mut case).

fn main():
    c: RefCell[i32] = RefCell(1)
    with _a = c.borrow_mut():
        with _b = c.borrow():
            println(1)
#$ panics: RefCell already mutably borrowed (borrowed at
