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
from std.sync import Atomic, Mutex

fn work(n: int) -> int:
    total = 0
    for i in range(n):
        total += i % 7
    return total

fn worker(hits: thread_shared Atomic[int], names: thread_shared Mutex[Array[String]], k: int) -> int:
    for i in range(10000):
        hits.fetch_add(1)
    with g = names.lock():
        g.push(owns "worker".to_string())
    return work(1000 + k)

fn main():
    hits = thread_shared Atomic(0)
    names: thread_shared Mutex[Array[String]] = thread_shared Mutex([])
    handles: Array[thread.JoinHandle[int]] = []
    for k in range(4):
        mine = thread_shared hits
        theirs = thread_shared names
        handles.push(owns thread.spawn(owns owned fn() => worker(mine, theirs, k)))
    total = 0
    while handles.len() > 0:
        match handles.pop():
            Some(h):
                total += h.join()
            None:
                pass
    println(hits.load(), total)
    with g = names.lock():
        println(g.len())
    quiet = thread.spawn(owns owned fn() => work(5))
    println(quiet.join())
    done = thread.spawn(owns owned fn() => println("from a thread"))
    done.join()
    text = thread.spawn(owns owned fn() => "made".to_string() + " here")
    println(text.join())
    _detached = thread.spawn(owns owned fn() => work(10))
    _also = thread.spawn(owns owned fn() => "dropped by its own thread".to_string())
