#$ test: run-pass
#$ rules: ECS-4
#$ profiles: debug, development
#$ stdout: 311 311
#$ 2
# `[ECS-4]` — `world.run_parallel([s1, s2, …])` runs systems whose read and write sets do not
# conflict: `movement` writes `Pos` and reads `Vel`, `regen` writes `Hp`; `report` and `audit`
# only read, so they may share `Pos` and `Hp`. Each system's sets are compile-time constants of
# its query's type (`Mut[T]` writes `T`, a plain `T` reads it), compared before any system runs.
# The systems run at once, on threads (ODR-218), so the readers record what they saw for `main`
# to print: two threads printing at once may interleave.
from std.ecs import World, Query, Mut, Component, System
from std.sync import Atomic

static REPORTED: Atomic[int] = Atomic(0)
static AUDITED: Atomic[int] = Atomic(0)

@derive(Copy, Component)
struct Pos:
    x: int

@derive(Copy, Component)
struct Vel:
    x: int

@derive(Copy, Component)
struct Hp:
    n: int

fn movement(mut q: Query[(Mut[Pos], Vel)]):
    for p, v in q.iter_mut():
        p.x += v.x

fn regen(mut q: Query[Mut[Hp]]):
    for h in q.iter_mut():
        h.n += 1

fn report(mut q: Query[(Pos, Hp)]):
    for p, h in q.iter():
        REPORTED.store(p.x * 100 + h.n)

fn audit(mut q: Query[(Hp, Pos)]):
    for h, p in q.iter():
        AUDITED.store(p.x * 100 + h.n)

fn main():
    w = World()
    e = w.create()
    w.add(e, Pos(1))
    w.add(e, Vel(2))
    w.add(e, Hp(10))
    w.run_parallel([movement, regen])
    w.run_parallel([report, audit])
    println(REPORTED.load(), AUDITED.load())
    systems: Array[System] = [movement, regen]
    println(systems.len())
