# Clause BRW-20.1: "A view made through a lock's guard (`g[0]`, `ref g[0]`, a slice of `g`) points
# into the guard and does not leave its `with` block (`E3063`"; "the same for `RwLock`'s `read()` and
# `write()` and for a `guarded by` field inside `with x.lock:`"
#$ test: compile-fail
#$ rules: BRW-20, THR-3, THR-18
# The spec's two messages; the `guarded by` field's view is code only.

from std.sync import Mutex, RwLock, Lock

class Log:
    lock: Lock = Lock()
    lines: Array[String] guarded by lock = []

fn first_locked(m: Mutex[Array[String]]) -> StrView:
    with g = m.lock():
        return g[0]    #$ error[E3063]: this points into the locked list, which can change once the lock ends; return a copy, or do the work inside the `with` block

fn first_read(rw: RwLock[Array[String]]) -> StrView:
    with g = rw.read():
        return g[0]    #$ error[E3063]: this points into the read-locked list, which can change once the lock ends; return a copy: `return copy of g[0]`, with the result type `String`

fn first_line(log: Log) -> StrView:
    with log.lock:
        return log.lines[0]    #$ error[E3063]

fn main():
    m = Mutex([String("a")])
    println(first_locked(m))
    rw = RwLock([String("b")])
    println(first_read(rw))
    println(first_line(Log()))
