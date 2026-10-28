#$ test: run-pass
#$ rules: BRW-16, CLO-1
#$ stdout: 2
## design rules: R8, R12
## R8 (c): the change goes after the walk: a system destroys, creates and adds through the query's
## `Commands`, applied after the system returns (as Bevy). AN-06's NEW program, with a `main`.

from std.ecs import World, Entity, Query, Component

@derive(Copy, Component)
struct Health:
    hp: int

fn reap(ref mut q: Query[(Entity, ref mut Health)], dt: float):
    for e, ref mut h in q:
        h.hp -= 1
        if h.hp <= 0:
            q.commands().destroy(e)
            n = q.commands().create()
            q.commands().add(n, Health(hp=10))

fn main():
    w = World()
    for n in [1, 3]:
        e = w.create()
        w.add(e, Health(hp=n))
    w.run(reap, 0.016)
    println(w.len())
