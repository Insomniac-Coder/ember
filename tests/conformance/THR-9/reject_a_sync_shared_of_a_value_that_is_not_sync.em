#$ test: compile-fail
#$ rules: THR-9, THR-8, HEAP-10
#$ error[E7001]: field `Holder.held` has type `SyncShared[Cell[i64]]`, which is not Send or Sync
# `[THR-9]` — `SyncShared[T]` is `Sync` only for a `T` that is `Send` and `Sync`: every owner's
# thread reads the same `T`. `[IDE-3]` — `main`, which no error is in, is checked on.

from std.sync import SyncShared

@sync
class Holder:
    held: SyncShared[Cell[int]]

fn main():
    h = Holder(SyncShared(Cell(1)))  #$ warning[L1001]: `h` is never read
