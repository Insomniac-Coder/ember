# Clause THR-20.2: "A `static mut` the tasks of a scope reach is a place `[THR-11]` checks as any
# other: two tasks changing it, or one reading what another changes, is `E7007`, other than by a
# reduction, under a lock or through an `Atomic`."
#$ test: compile-fail
#$ rules: THR-20, THR-11
#$ error[E7007]
# Two tasks give `FRAME` a value. No text in the spec: code only, claimed for the file (Q3).

import std.thread

static mut FRAME: int = 0

fn main():
    with scope = thread.scope():
        scope.spawn(owns fn(): FRAME = 1)
        scope.spawn(owns fn(): FRAME = 2)
    println(FRAME)
