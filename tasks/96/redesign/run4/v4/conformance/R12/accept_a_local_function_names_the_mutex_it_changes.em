#$ test: run-pass
#$ rules: R12, R5, R15
#$ stdout: 1
## Verdicts 29, 30 and 51, R12 (rules.md lines 605-608, 686-692): the local function names
## `Mutex[Stats]`, is passed to `thread.spawn` by its name, and `main`, which starts the thread, names
## the change too.

import std.thread
from std.sync import Mutex

struct Stats:
    total: int = 0

fn main() changes Mutex[Stats]:
    stats = thread_shared Mutex(Stats())
    fn add_one() with thread_shared stats changes Mutex[Stats]:
        with g = stats.lock():
            g.total += 1
    t = thread.spawn(add_one)
    t.join()
    with g = stats.lock():
        println(g.total)
