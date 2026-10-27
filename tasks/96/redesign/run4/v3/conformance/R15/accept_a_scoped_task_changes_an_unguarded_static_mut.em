#$ test: run-pass
#$ rules: R15, R5
#$ stdout: 1
## R15: a `static mut` "is reached only by the main thread and its scoped tasks": one task of a scope
## the main thread opens may change it, and it is read after the join.

import std.thread

static mut FRAME: int = 0

fn bump() changes FRAME:
    FRAME += 1

fn main() changes FRAME:
    with scope = thread.scope():
        scope.spawn(fn() changes FRAME: bump())
    println(FRAME)
