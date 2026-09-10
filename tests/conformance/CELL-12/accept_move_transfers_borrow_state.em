#$ test: run-pass
#$ rules: CELL-12
# `[CELL-12]` (S3/ADR-021, owner ruling) — moving a `RefCell` transfers the
# whole cell, borrow state included. `d` takes `c`'s value and its `0`
# counter; borrowing `d` then succeeds.

fn main():
    c: RefCell[i32] = RefCell(1)
    d = c
    with _g = d.borrow():
        println(1)
#$ stdout: 1
