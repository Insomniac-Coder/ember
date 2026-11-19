#$ test: compile-fail
#$ rules: THR-9, THR-1
# `[THR-9]` — readers of an `RwLock[T]` hold `ref T` on several threads at once, so it is
# `ThreadShared` only when `T` is: `RwLock[Cell[int]]` is not, though `Mutex[Cell[int]]` is.

from std.sync import RwLock

class Holder:
    held: RwLock[Cell[int]] = RwLock(Cell(0))

fn main():
    _h = thread_shared Holder()    #$ error[E7015]: `Holder.held` is a `std.sync.RwLock[Cell[i64]]`, which several threads may not read at once
