#$ test: run-fail
#$ rules: ECS-4, EXC-18
#$ profiles: debug, development
#$ panics: exclusivity violation: write access to std.ecs.AosStorage_Hp
# `[ECS-4]` — the run holds the storages its query names for the whole of the system: `Hp` is
# written through the query, so writing it another way (here the world, given to the system as
# its argument) while the query lives is the exclusivity violation of `[EXC-1]`, as for any
# field a live view holds (`[EXC-18]`).
from std.ecs import World, Query, Mut, Component, Entity

@derive(Copy, Component)
struct Hp:
    n: int

fn reset(mut q: Query[(Entity, Mut[Hp])], w: World):
    for e, hp in q.iter_mut():
        hp.n += 1
        w.add(e, Hp(5))

fn main():
    w = World()
    e = w.create()
    w.add(e, Hp(1))
    w.run(reset, w)
