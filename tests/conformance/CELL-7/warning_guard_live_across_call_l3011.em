#$ test: run-pass
#$ rules: CELL-7
# `[CELL-7]` — `L3011` fires (not blocks, not opt-in) when a guard is live
# across a call that could re-enter the same cell. Here the `println` runs
# with the shared guard live; the lint warns and the program still runs.
#$ warning[L3011]: live across this call

fn main():
    c: RefCell[i32] = RefCell(1)
    with _g = c.borrow():
        println(1)
#$ stdout: 1
