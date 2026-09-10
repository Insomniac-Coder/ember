#$ test: run-pass
#$ rules: CELL-7
# `[CELL-7]` — nested borrows and nested scopes. Two shared guards nest; both
# drop in reverse order at their block ends, and a later exclusive borrow
# succeeds.

fn main():
    c: RefCell[i32] = RefCell(1)
    with _a = c.borrow():
        with _b = c.borrow():
            println(1)
    println(2)
    with _m = c.borrow_mut():
        println(3)
#$ stdout: 1
#$ 2
#$ 3
