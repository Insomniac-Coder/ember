#$ test: run-pass
#$ rules: R15, R12, R5
#$ stdout: 3
## R15 (verdict 77): the first way out E7007 gives: the field is an `Atomic`, read and added to by its
## own operations from any task (R5); the task names `stats`, the variable it captured, as it changes it
## (verdict 75).

import std.thread
from std.sync import Atomic

class Stats:
    total: Atomic[int] = Atomic(0)

fn add_small(xs: Array[int]) -> int:
    stats = Stats()
    with scope = thread.scope():
        for x in xs:
            fn add() with copy of x changes stats:
                if stats.total.load() < 10:
                    stats.total.fetch_add(x)
            scope.spawn(add)
    return stats.total.load()

fn main():
    println(add_small([1, 2]))
