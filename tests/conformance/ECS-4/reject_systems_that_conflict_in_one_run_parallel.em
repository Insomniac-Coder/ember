#$ test: compile-fail
#$ rules: ECS-4
# `[ECS-4]` — "a conflict between two systems known at compile time is `E7020`, naming both
# systems and the component": `movement` writes `Pos` and `report` reads it; `regen` writes `Hp`,
# which `report` reads. `movement` and `regen` share nothing and would run at once.
from std.ecs import World, Query, Mut, Component

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
        println(p.x, h.n)

fn main():
    w = World()
    w.run_parallel([movement, report, regen])
#$ error[E7020]: systems `movement` and `report` cannot run at once: `movement` writes `Pos`, which the other uses
#$ error[E7020]: systems `report` and `regen` cannot run at once: `regen` writes `Hp`, which the other uses
