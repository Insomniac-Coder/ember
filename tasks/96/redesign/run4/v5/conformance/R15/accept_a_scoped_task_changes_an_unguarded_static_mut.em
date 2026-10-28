#$ test: run-pass
#$ rules: THR-1, FN-4
#$ stdout: 1
## design rules: R15, R5
## R15: a `static mut` "is reached only by the main thread and its scoped tasks": one task of a scope
## the main thread opens may change it, and it is read after the join.

import std.thread

static mut FRAME: int = 0

fn bump():
    FRAME += 1

fn main():
    with scope = thread.scope():
        scope.spawn(fn(): bump())
    println(FRAME)
