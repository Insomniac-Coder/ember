#$ test: run-pass
#$ profiles: debug, release, shipping
#$ rules: CELL-6a, CELL-6, PRF-1
# `[CELL-6a]` is quantified over every profile. Both fallible operations must
# retain their `None` contention path in debug, release, and shipping; neither
# profile optimization nor unchecked-overflow policy may make them infallible.

fn main():
    cell: RefCell[i32] = RefCell(1)
    with _shared = cell.borrow():
        match cell.try_borrow_mut():
            Some(_guard):
                println(0)
            None:
                println(1)
    with _mutable = cell.borrow_mut():
        match cell.try_borrow():
            Some(_guard):
                println(0)
            None:
                println(2)
#$ stdout: 1
#$ 2
