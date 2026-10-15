#$ test: run-pass
#$ rules: THR-11, THR-5
#$ stdout: 8
#$ after
# `[THR-11]` — `h.join()` waits early; a function may be a task; a task nobody joins is joined at
# the block's end (`[THR-5]`).

import std.thread

fn work() -> int:
    return 4

fn main():
    with scope = thread.scope():
        a = scope.spawn(fn() => work())
        b = scope.spawn(work)
        println(a.join() + b.join())
        _c = scope.spawn(fn() => work() * 10)
    println("after")
