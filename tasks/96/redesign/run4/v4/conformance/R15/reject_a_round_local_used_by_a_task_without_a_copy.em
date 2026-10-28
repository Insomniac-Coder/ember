#$ test: compile-fail
#$ rules: R15, R12
## R15: "A local of one loop round is given to a task with `with copy of n`"; without it the task may
## outlive the round's `n` (E3026, AN-73).

import std.thread
from std.sync import Atomic

fn count(texts: Array[String]) -> int:
    total = Atomic(0)
    with scope = thread.scope():
        for t in texts:
            n = t.split_whitespace().count()
            scope.spawn(fn() changes total: total.fetch_add(n))    #$ error[E3026]: the task uses `n`, which ends at the end of this round of the loop (line 13), while the task may still be running; give the task its own copy: `fn() with copy of n changes total:`
    return total.load()

fn main():
    println(count([String("a b"), String("c")]))
