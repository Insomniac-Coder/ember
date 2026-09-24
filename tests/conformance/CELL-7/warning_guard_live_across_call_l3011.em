#$ test: run-pass
#$ rules: CELL-7
#$ stdout: 1
# `[CELL-7]` — `L3011` fires (not blocks, not opt-in) when a guard is live
# across a call that may reach the same cell; a function the compiler cannot
# see into is such a call. The lint warns and the program still runs.
# `println` provably cannot reach the cell, so it never warns (F-187).

fn tick() -> int:
    return 1

fn main():
    c: RefCell[i32] = RefCell(1)
    with _g = c.borrow():
        println(tick())    #$ warning[L3011]: live across this call
