#$ test: run-pass
#$ rules: THR-9, THR-1, THR-3
#$ stdout: 3
# `[THR-9]` — `Mutex[T]` asks only that `T` be `Send`: one thread at a time holds the `ref mut T`, so
# a `Mutex[Cell[int]]` is `Sync` where an `RwLock[Cell[int]]` is not.

from std.sync import Mutex

@sync
class Counter:
    count: Mutex[Cell[int]] = Mutex(Cell(0))

fn main():
    c = Counter()
    with g = c.count.lock():
        g.set(g.get() + 3)
    with g = c.count.lock():
        println(g.get())
