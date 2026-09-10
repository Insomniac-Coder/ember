#$ test: run-pass
#$ rules: CELL-6
# `[CELL-6]` — `try_borrow` gives `Some` when free. The guard is bound by
# `with` inside the `Some` arm (`[CELL-7]`); matching consumes the `Option`
# without panicking.

fn main():
    c: RefCell[i32] = RefCell(1)
    m = c.try_borrow()
    match m:
        Some(_g):
            println(1)
        None:
            println(2)
#$ stdout: 1
