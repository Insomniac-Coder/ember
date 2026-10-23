#$ test: run-pass
#$ profiles: debug, development, release
#$ rules: CELL-9, CELL-5
#$ assert-c: contains("em_3std4core10UnsafeCellG9ee56c8f11b4d4af borrow;")
# `[CELL-9]` requires the borrow-state counter itself to occupy one machine
# word. Source-location fields used by `[CELL-5]` are separate metadata and do
# not change that counter representation. Generated C pins this independently
# in each profile: the counter `borrow` is an `UnsafeCell[isize]` (ODR-275: what
# changes through a shared `RefCell` is kept in an `UnsafeCell`), whose one
# member is a `ptrdiff_t`: its C name's `G9ee56c8f11b4d4af` is `isize`'s.

fn main():
    cell: RefCell[i32] = RefCell(7)
    with value = cell.borrow():
        println(value)
#$ stdout: 7
