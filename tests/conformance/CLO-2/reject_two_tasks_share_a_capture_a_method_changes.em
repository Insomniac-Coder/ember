#$ test: compile-fail
#$ rules: CLO-2, EXC-15, THR-8
#$ error[E3022]: `r` is already mutably borrowed
# `[CLO-2]`, `[EXC-15]` — a lambda's capture is read-only unless its body changes it, and what a
# method does to its object is worked out from its body: `tick` adds to the `Count` it is called
# on, so each task captures `r` to change it, and two at once are refused.

import std.thread

interface Tick: needs (Send, Sync):
    fn tick() -> int

struct Count:
    n: int

extend Count implements Tick:
    fn tick() -> int:
        this.n += 1
        return this.n

fn main():
    r: Box[dyn Tick] = Box(Count(0))
    with scope = thread.scope():
        a = scope.spawn(fn() => r.tick())
        b = scope.spawn(fn() => r.tick())
        println(a.join() + b.join())
