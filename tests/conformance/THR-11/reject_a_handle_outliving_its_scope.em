#$ test: compile-fail
#$ rules: THR-11, THR-5
#$ error[E3060]: `scope` does not live long enough
# `[THR-11]` — a scoped task's handle borrows its scope, whose end joins and frees the thread.

import std.thread

fn work() -> int:
    return 4

fn leak() -> thread.ScopedJoinHandle[int]:
    with scope = thread.scope():
        return scope.spawn(fn() => work())

fn main():
    println(leak().join())
