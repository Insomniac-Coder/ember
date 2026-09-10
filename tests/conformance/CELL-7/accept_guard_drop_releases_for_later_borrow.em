#$ test: run-pass
#$ rules: CELL-7
# `[CELL-7]` — a guard's `drop` releases the borrow state. The first `borrow`
# ends at its `with` block exit; the later `borrow_mut` then succeeds. Without
# the release the second borrow would panic.

fn main():
    c: RefCell[i32] = RefCell(1)
    with _a = c.borrow():
        println(1)
    with _b = c.borrow_mut():
        println(2)
#$ stdout: 1
#$ 2
