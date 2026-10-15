#$ test: compile-fail
#$ rules: THR-9, THR-1
#$ error[E7001]: `std.sync.RwLock[Cell[i64]]`, which is not Sync
# `[THR-9]` — readers of an `RwLock[T]` hold `ref T` on several threads at once, so it is `Sync`
# only when `T` is: `RwLock[Cell[int]]` is not, though `Mutex[Cell[int]]` is.

from std.sync import RwLock

@sync
class Holder:
    held: RwLock[Cell[int]] = RwLock(Cell(0))

fn main():
    h = Holder()
