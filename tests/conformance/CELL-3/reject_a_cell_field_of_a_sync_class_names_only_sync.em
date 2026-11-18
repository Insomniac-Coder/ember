#$ test: compile-fail
#$ rules: CELL-3, THR-1
## `[CELL-3]` — `Cell[T]` may move between threads when `T: Send`, but is never shared: the message says
## that several threads may not read it at once, not that it stays on one.

class Shared:
    let cell: Cell[int]

    fn init():
        this.cell = Cell(1)

fn main():
    _s = thread_shared Shared()    #$ error[E7015]: `Shared.cell` is a `Cell[i64]`, which several threads may not read at once
