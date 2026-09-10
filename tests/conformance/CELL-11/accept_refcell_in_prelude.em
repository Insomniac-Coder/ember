#$ test: run-pass
#$ rules: CELL-11
# `[CELL-11]` (`OQ-10`) — `RefCell` is in the prelude: the name resolves with
# no import, like `Cell`, `Array` and `Option` do for compiler-known types.

fn main():
    c: RefCell[i32] = RefCell(7)
    with _g = c.borrow():
        println(1)
#$ stdout: 1
