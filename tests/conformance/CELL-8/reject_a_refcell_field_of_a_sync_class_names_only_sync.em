#$ test: compile-fail
#$ rules: CELL-8, THR-1
## `[CELL-8]` — `RefCell[T]` is not `ThreadShared` and, like any type, `Send` when what it holds is. (The
## cell's private record of where a borrow began is not part of what it holds.)

class Shared:
    let cell: RefCell[int]

    fn init():
        this.cell = RefCell(1)

fn main():
    _s = thread_shared Shared()    #$ error[E7015]: `Shared.cell` is a `RefCell[i64]`, which several threads may not read at once
