#$ test: run-pass
#$ rules: R15, R12, R5
#$ stdout: 3
## R15: "A local of one loop round is given to a task with `with copy of n`"; two tasks changing one
## `Atomic` is not E7007, the atomic instruction orders each change (AN-73's NEW).

import std.thread
from std.sync import Atomic

fn count(texts: Array[String]) -> int:
    total = Atomic(0)
    with scope = thread.scope():
        for t in texts:
            n = t.split_whitespace().count()
            scope.spawn(fn() with copy of n changes total: total.fetch_add(n))
    return total.load()

fn main():
    println(count([String("a b"), String("c")]))
