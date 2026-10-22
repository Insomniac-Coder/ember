#$ test: run-pass
#$ rules: ECS-4, STD-19, BRW-2
#$ profiles: debug, development, release
#$ stdout: 2 2 4
#$ stdout: 7
# `[STD-19]` (ODR-240), `[ECS-4]` — `world.run` borrows a named system as every std function
# borrows a named callable: a closure that counts the entities it saw (`seen += 1`) is lent with
# leave to change what it captured, run twice, and its count is the caller's to read after its
# last use. A lambda written at the call is given to the run, and what it changes is the
# caller's once the run returns.
from std.ecs import World, Query, Component

@derive(Copy, Component)
struct Hp:
    n: int

fn main():
    w = World()
    for n in [4, 5]:
        e = w.create()
        w.add(e, Hp(n))
    seen = 0
    runs = 0
    count = fn(mut q: Query[Hp], step: int):
        runs += step
        for _ in q.iter():
            seen += 1
    w.run(count, 1)
    w.run(count, 1)
    println(runs, w.len(), seen)
    w.run(fn(mut q: Query[Hp], step: int): runs += step, 5)
    println(runs)
