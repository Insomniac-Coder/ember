#$ test: run-pass
#$ rules: ECS-4, THR-8, CLO-10
#$ profiles: debug, development, release
#$ stdout: 3
## `[ECS-4]` (D-899) — two lambda systems of one `run_parallel` may capture the same `@sync`
## object, after another `run_parallel` of named systems. The evaluator stopped at the first run
## ("invalid downcast" fetching `Pos`'s storage: it read the storage's handle by its low byte); the
## compiled program was right.

from std.ecs import World, Query, Mut, Component, Entity, system
from std.sync import Atomic

@derive(Copy, Component)
struct Pos:
    x: int

@derive(Copy, Component)
struct Vel:
    x: int

fn left(mut q: Query[(Entity, Pos)]):
    pass

fn right(mut q: Query[(Entity, Mut[Vel])]):
    pass

@sync
class Tally:
    total: Atomic[int] = Atomic(0)

fn main():
    w = World()
    a = w.create()
    w.add(a, Pos(3))
    b = w.create()
    w.add(b, Vel(4))
    w.run_parallel([left, right])
    tally = Tally()
    counted = system(owned fn(mut q: Query[Vel]): tally.total.fetch_add(1), "counted")
    w.run_parallel([counted, system(owned fn(mut q: Query[Pos]): tally.total.fetch_add(2), "two")])
    println(tally.total.load())
