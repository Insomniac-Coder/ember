#$ test: run-pass
#$ rules: THR-10, THR-8, HEAP-10, THR-14, THR-3
#$ profiles: debug, development
#$ stdout: 40000 12007
#$ 4
#$ 10
#$ from a thread
#$ made here
# `[THR-10]` — `thread.spawn(owned fn() => …)` runs the function on a new thread; `join` waits and gives
# what it returned (`void`, an `int`, a `String`); a handle dropped unjoined detaches its thread, which
# runs on and drops its own result. Four workers share an `Atomic` and a `Mutex` through
# `SyncShared` (`[THR-8]`: what a thread captures is `Send`).

import std.thread
from std.sync import Atomic, Mutex, SyncShared

fn work(n: int) -> int:
    total = 0
    for i in range(n):
        total += i % 7
    return total

fn worker(hits: SyncShared[Atomic[int]], names: SyncShared[Mutex[Array[String]]], k: int) -> int:
    for i in range(10000):
        hits.get().fetch_add(1)
    with g = names.get().lock():
        g.push("worker".to_string())
    return work(1000 + k)

fn main():
    hits = SyncShared(Atomic(0))
    names: SyncShared[Mutex[Array[String]]] = SyncShared(Mutex([]))
    handles: Array[thread.JoinHandle[int]] = []
    for k in range(4):
        mine = hits
        theirs = names
        handles.push(thread.spawn(owned fn() => worker(mine, theirs, k)))
    total = 0
    while handles.len() > 0:
        match handles.pop():
            Some(h):
                total += h.join()
            None:
                pass
    println(hits.get().load(), total)
    with g = names.get().lock():
        println(g.len())
    quiet = thread.spawn(owned fn() => work(5))
    println(quiet.join())
    done = thread.spawn(owned fn() => println("from a thread"))
    done.join()
    text = thread.spawn(owned fn() => "made".to_string() + " here")
    println(text.join())
    _detached = thread.spawn(owned fn() => work(10))
    _also = thread.spawn(owned fn() => "dropped by its own thread".to_string())
