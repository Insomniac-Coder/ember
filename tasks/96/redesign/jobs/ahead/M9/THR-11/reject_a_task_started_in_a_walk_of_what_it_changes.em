# Clause THR-11.2: "A task started inside a walk of what it changes is `E3020` (...), not `E7007`,
# whose fix would leave the walk refused."
#$ test: compile-fail
#$ rules: THR-11, BRW-18, DIA-28
# The spec's message, with this file's line numbers.

import std.thread

fn main():
    names = [String("a")]
    with scope = thread.scope():
        for n in names:
            scope.spawn(owns fn(): names.push(owns String("x")))    #$ error[E3020]: `scope.spawn(...)` (line 13) starts a task that pushes to `names` (`fn(): names.push(owns String("x"))`), which this loop walks (line 12); start it after the loop: move line 13 below the loop
            println(n)
