#$ test: run-pass
#$ profiles: debug, release, shipping
#$ rules: CELL-9, CELL-5
#$ assert-c: contains("ptrdiff_t borrow;")
# `[CELL-9]` requires the borrow-state counter itself to occupy one machine
# word. Source-location fields used by `[CELL-5]` are separate metadata and do
# not change that counter representation. Generated C pins this independently
# in each profile.

fn main():
    cell: RefCell[i32] = RefCell(7)
    with value = cell.borrow():
        println(value)
#$ stdout: 7
