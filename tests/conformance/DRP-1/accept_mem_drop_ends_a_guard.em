#$ test: run-pass
#$ rules: DRP-1, CELL-5
#$ stdout: 7
# `[DRP-1]` — `mem.drop(g)` ends a guard's life where it is written, so the `RefCell` borrow it
# held is over (D-594: the guard was read through, and its borrow lasted to the end of the scope).

fn main():
    c = RefCell(5)
    g = c.borrow_mut()
    mem.drop(g)
    h = c.borrow_mut()
    h += 2
    mem.drop(h)
    println(c.borrow())
