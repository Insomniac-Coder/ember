#$ test: run-pass
#$ rules: THR-3, CELL-8, DRP-6
#$ profiles: debug, release
#$ stdout: true
#$ 15
#$ 2 b
#$ free 15
# `[THR-3]` — `Mutex[T]` is a value type; `m.lock()` gives a `MutexGuard[T]` that reads through to
# `ref mut T`, and the lock is released when the guard ends: at the end of `with g = m.lock():`,
# or where `mem.drop` ends it (`[DRP-6]`). `try_lock` never waits: `None` while the lock is held.
# The shape is `RefCell`'s (`[CELL-8]`).

from std.sync import Mutex

fn main():
    m = Mutex(5)
    with g = m.lock():
        g += 10
    tags: Mutex[Array[String]] = Mutex([])
    with t = tags.lock():
        t.push("a".to_string())
        t.push("b".to_string())
    held = m.lock()
    println(m.try_lock().is_none())
    mem.drop(held)
    with g = m.lock():
        println(g)
    with t = tags.lock():
        println(t.len(), t[1])
    match m.try_lock():
        Some(g):
            println("free", g)
        None:
            println("held")
