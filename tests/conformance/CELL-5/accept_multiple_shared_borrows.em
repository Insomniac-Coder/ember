#$ test: run-pass
#$ rules: CELL-5
# `[CELL-5]` — `borrow` succeeds unless a mutable borrow is active. Two shared
# guards may be live at once; the counter holds `2` across the inner block.
# Bound by `with` (`[CELL-7]`); the `println` calls lint `L3011` (a guard live
# across a call) but do not fail.

fn main():
    c: RefCell[i32] = RefCell(1)
    with _a = c.borrow():
        with _b = c.borrow():
            println(1)
    println(2)
#$ stdout: 1
#$ 2
#$ assert-c: contains("panic_refcell")
#$ assert-c: contains("->borrow")
