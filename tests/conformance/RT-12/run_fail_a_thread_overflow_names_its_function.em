#$ test: run-fail
#$ rules: RT-12, THR-10
#$ profiles: debug
#$ panics: stack overflow in climb
# `[RT-12]` — every thread the runtime creates has a guard page and the same handler: an overflow
# on a spawned thread aborts naming its function too.

import std.thread

fn climb(n: int) -> int:
    near = [n, n + 1, n + 2, n + 3]
    r = climb(n + 1)
    return r + near[r % 4]

fn main():
    worker = thread.spawn(owned fn() => climb(0))
    println(worker.join())
