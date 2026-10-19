#$ test: run-pass
#$ rules: THR-3, CELL-8, DRP-6
#$ profiles: debug, development
#$ stdout: 15 15 30
#$ true true
#$ wrote 16
#$ 16
# `[THR-3]` — `RwLock[T]` gives read guards, which read through to `ref T` and which several
# holders share, and write guards, which read through to `ref mut T` and which one holder has
# alone; each guard's end releases its share (`[DRP-6]`). `try_write` gives `None` while a reader
# holds the lock, `try_read` gives a guard. The shape is `RefCell`'s (`[CELL-8]`).

from std.sync import RwLock

fn main():
    rw = RwLock(5)
    with w = rw.write():
        w += 10
    a = rw.read()
    b = rw.read()
    println(a, b, a + b)
    println(rw.try_write().is_none(), rw.try_read().is_some())
    mem.drop(a)
    mem.drop(b)
    match rw.try_write():
        Some(w):
            w += 1
            println("wrote", w)
        None:
            println("held")
    with r = rw.read():
        println(r)
