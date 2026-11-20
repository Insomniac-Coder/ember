# Clause THR-11.1: "After `h.join()` the parent may change or read what that task used, in the same
# block (`counts[0] += 1` after `h.join()`), checked only against the tasks not yet joined"
# (verdict 104)
#$ test: run-pass
#$ rules: THR-11, THR-12
#$ stdout: 6
#$ stdout: 2
# The parent changes `counts[0]` after the task that changed it is joined; a task still running
# changes another place.

import std.thread

fn main():
    counts = [0, 0]
    with scope = thread.scope():
        h = scope.spawn(owns fn(): counts[0] = 5)
        scope.spawn(owns fn(): counts[1] = 2)
        h.join()
        counts[0] += 1
    println(counts[0])
    println(counts[1])
