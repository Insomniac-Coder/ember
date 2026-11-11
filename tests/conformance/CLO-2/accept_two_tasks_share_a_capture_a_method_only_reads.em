#$ test: run-pass
#$ rules: CLO-2, EXC-15, THR-8
#$ stdout: 0
# `[CLO-2]`, `[EXC-15]` — a lambda's capture is read-only unless its body changes it, and what a
# method does to its object is worked out from its body: `tick` only reads the `Count` it is called
# on, so both tasks capture `r` read-only and may run at once.

import std.thread

interface Tick: needs (Send, Sync):
    fn tick() -> int

struct Count:
    n: int

extend Count implements Tick:
    fn tick() -> int:
        return this.n

fn main():
    r: owns dyn Tick = owns Count(0)
    with scope = thread.scope():
        a = scope.spawn(owns fn() => r.tick())
        b = scope.spawn(owns fn() => r.tick())
        println(a.join() + b.join())
