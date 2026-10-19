#$ test: run-fail
#$ rules: ECS-4
#$ profiles: debug, development
#$ stdout: before
#$ panics: systems `movement` and `report` cannot run at once: `movement` writes `Pos`, which the other uses
# `[ECS-4]` — "Systems passed as values, whose sets are not known at compile time, are checked
# against each other before any of them starts, and a conflict panics, naming both systems and
# the component": the array is built first and given to `run_parallel` as a value, so the check
# is made when it runs, and `regen`, first in the array, does not run.
from std.ecs import World, Query, Mut, Component, System

@derive(Copy, Component)
struct Pos:
    x: int

@derive(Copy, Component)
struct Hp:
    n: int

fn regen(mut q: Query[Mut[Hp]]):
    for h in q.iter_mut():
        h.n += 1
        println("regen ran")

fn movement(mut q: Query[Mut[Pos]]):
    for p in q.iter_mut():
        p.x += 1

fn report(mut q: Query[Pos]):
    for p in q.iter():
        println(p.x)

fn main():
    w = World()
    e = w.create()
    w.add(e, Pos(1))
    w.add(e, Hp(10))
    systems: Array[System] = [regen, movement, report]
    println("before")
    w.run_parallel(systems)
