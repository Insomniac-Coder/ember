#$ test: compile-fail
#$ rules: CELL-8, STA-1, THR-1
## `[CELL-8]` — `RefCell[T]` is `!Sync`: its borrow counter is not atomic, so a
## `static` may not hold one. `Mutex[T]` and `RwLock[T]` are the `Sync`
## equivalents.

static counter: RefCell[int] = RefCell(0) #$ error[E7002]: static type `RefCell[i64]` is not Sync

fn main():
    pass
