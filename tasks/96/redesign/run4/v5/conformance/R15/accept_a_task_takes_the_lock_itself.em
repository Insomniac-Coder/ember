#$ test: run-pass
#$ rules: THR-1, CLO-1, FN-1, FN-4
#$ stdout: 3
## design rules: R15, R12, R4, R5
## R15: the lines E7008 gives: the task takes the lock itself, after the parent's `with g` block ends; a
## `Mutex` takes no word at the call (R4); what `add_one`, the task and `main` change is worked out, and
## no first line says it (R5, verdict 86).

import std.thread
from std.sync import Mutex

fn add_one(m: Mutex[Array[int]]):
    with g = m.lock():
        g.push(1)

fn main():
    m = Mutex([0])
    with scope = thread.scope():
        with g = m.lock():
            g.push(2)
            scope.spawn(fn(): add_one(m))
    with g = m.lock():
        println(g.len())
