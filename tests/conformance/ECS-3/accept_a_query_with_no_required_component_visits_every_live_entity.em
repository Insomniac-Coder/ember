#$ test: run-pass
#$ rules: ECS-3, ECS-6
#$ profiles: debug, development
#$ stdout: 0 tagged
#$ 1 bare
#$ 2 tagged
#$ ---
#$ 0 tagged
#$ 2 tagged
#$ 1 bare
#$ 1
# `[ECS-3]` (ODR-142) — a query none of whose parameters requires a component, here `Entity` and
# an `Option`, has no smallest storage to drive it: it visits every live entity, in the world's
# order (`[ECS-6]`: creation order, a destroyed entity's hole filled by the last one). `d` takes
# the destroyed `b`'s index at its next generation, and comes last.
from std.ecs import World, Query, Entity, Component

@derive(Copy, Component)
struct Tag:
    on: bool

fn show(mut q: Query[(Entity, Option[Tag])], _: ()):
    for e, tag in q.iter():
        match tag:
            Some(_):
                println(e.index(), "tagged")
            None:
                println(e.index(), "bare")

fn main():
    world = World()
    a = world.create()
    b = world.create()
    c = world.create()
    world.add(a, Tag(on=true))
    world.add(c, Tag(on=true))
    world.run(show, ())
    println("---")
    _gone = world.destroy(b)
    d = world.create()
    world.run(show, ())
    println(d.generation())
