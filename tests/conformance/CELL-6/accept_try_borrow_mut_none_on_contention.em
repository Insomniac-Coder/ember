#$ test: run-pass
#$ rules: CELL-6, CELL-6a
# `[CELL-6]` — `try_borrow_mut` returns `None` while a shared borrow is live.
# `[CELL-6a]` as above: the `None` arm runs in every profile.

fn main():
    c: RefCell[i32] = RefCell(1)
    with _a = c.borrow():
        m = c.try_borrow_mut()
        match m:
            Some(_g):
                println(1)
            None:
                println(2)
#$ stdout: 2
