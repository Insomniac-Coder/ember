#$ test: compile-fail
#$ rules: CELL-3, STA-1, THR-1
## `[CELL-3]` — `Cell[T]` is `!Sync`: a `static` is shared by every thread, so
## it may not hold one. `Atomic[T]` is the `Sync` equivalent.

static counter: Cell[int] = Cell(0) #$ error[E7002]: static type `Cell[i64]` is not Sync

fn main():
    pass
