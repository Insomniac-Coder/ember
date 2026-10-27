#$ test: compile-fail
#$ rules: R15, R12
## R15: "a local of one round, which ends with it, [is] given to a task or a thread with a word"; without
## it the task may outlive the round's `n` (E3026, AN-73). `ref n` would not do either (the task may
## outlive the round), so the message gives only `copy of n`, with the line to write (R18). The nearest
## allowed program is `accept_a_task_takes_a_copy_of_a_round_local.em`.

import std.thread
from std.sync import Atomic

fn count(texts: Array[String]) -> int:
    total = Atomic(0)
    with scope = thread.scope():
        for t in texts:
            n = t.split_whitespace().count()
            scope.spawn(fn(): total.fetch_add(n))    #$ error[E3026]: the task uses `n`, which ends at the end of this round of the loop (line 15), while the task may still be running; give the task its own copy: write `scope.spawn(fn() copy of n: total.fetch_add(n))` in place of line 16
    return total.load()

fn main():
    println(count([String("a b"), String("c")]))
