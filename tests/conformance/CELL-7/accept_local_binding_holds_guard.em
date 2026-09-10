#$ test: run-pass
#$ rules: CELL-7
# `[CELL-7]` — guards `MUST be bound by with or a local`. A `let` binding holds
# the guard to the end of its block; the inner block's guard drops at the
# block end, so the later `borrow_mut` succeeds.

fn main():
    c: RefCell[i32] = RefCell(1)
    if true:
        g = c.borrow()
        println(1)
    with _b = c.borrow_mut():
        println(2)
#$ stdout: 1
#$ 2
