#$ test: run-fail
#$ rules: ECS-5, ECS-4
#$ profiles: debug, development
#$ panics: exclusivity violation: write access to std.ecs.AosStorage_Hp.entities
# `[ECS-5]` — why a system destroys through its `Commands`: destroying an entity removes it from
# every storage, moving another entity into its slot, and the query iterating `Hp` holds that
# storage (`[ECS-4]`). Done through the world while the query lives, it panics; through
# `q.commands()` it is made after the system returns.
from std.ecs import World, Query, Component, Entity

@derive(Copy, Component)
struct Hp:
    n: int

fn cull(mut q: Query[(Entity, Hp)], w: World):
    for e, hp in q.iter():
        if hp.n <= 0:
            w.destroy(e)

fn main():
    w = World()
    e = w.create()
    w.add(e, Hp(0))
    w.run(cull, w)
