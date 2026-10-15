# Two threads each take one `Mutex` two million times to add one under it (`[THR-3]`): the cost of
# the lock when the other thread holds it as often as not. The C twin is the same with the
# platform's own lock (SRWLOCK on Windows, a pthread mutex elsewhere).

import std.thread
from std.sync import Mutex, SyncShared

fn bump(m: SyncShared[Mutex[int]], n: int):
    for i in range(n):
        with g = m.get().lock():
            g += 1

fn main():
    n = 2_000_000
    m = SyncShared(Mutex(0))
    theirs = m
    other = thread.spawn(owned fn() => bump(theirs, n))
    bump(m, n)
    other.join()
    with g = m.get().lock():
        println(g)
