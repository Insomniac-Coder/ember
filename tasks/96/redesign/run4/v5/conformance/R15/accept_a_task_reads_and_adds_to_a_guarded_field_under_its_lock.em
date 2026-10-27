#$ test: run-pass
#$ rules: R15, R12
#$ stdout: 3
## R15 (verdict 77; F-10): the lines E7007 gives a task that reads the place before it adds: the field is
## guarded by its object's lock, and `add_below` reads and adds inside it, so no task sees a total
## another is changing and the total never passes 10. It replaces step 3's
## `accept_a_task_reads_an_atomic_field_another_task_adds_to.em`, whose two separate atomic steps let
## two tasks both read 9 and both add (F-10: E7007 no longer offers it).

import std.thread
from std.sync import Lock

class Stats:
    lock: Lock = Lock()
    total: int guarded by lock = 0

    fn add_below(limit: int, n: int):
        with this.lock:
            if this.total < limit:
                this.total += n

fn add_small(xs: Array[int]) -> int:
    stats = Stats()
    with scope = thread.scope():
        for x in xs:
            fn add() copy of x:
                stats.add_below(10, x)
            scope.spawn(add)
    with stats.lock:
        return stats.total

fn main():
    println(add_small([1, 2]))
