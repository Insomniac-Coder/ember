#$ test: run-pass
#$ rules: CLO-1, FN-4, THR-1
#$ stdout: 1
## design rules: R12, R5, R15
## Verdicts 51 and 86, R12 (rules.md lines 674-676: "a local function captures as a lambda does, so a
## long lambda may be written inside the call or as a named local function passed by its name"), R15
## (lines 840-842): the local function holds a `thread_shared` owner of the `Mutex` and changes the
## `Stats` inside it under its lock; it is passed to `thread.spawn` by its name, and nothing on any first
## line says what it changes.

import std.thread
from std.sync import Mutex

struct Stats:
    total: int = 0

fn main():
    stats = thread_shared Mutex(Stats())
    fn add_one() thread_shared stats:
        with g = stats.lock():
            g.total += 1
    t = thread.spawn(add_one)
    t.join()
    with g = stats.lock():
        println(g.total)
