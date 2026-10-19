#$ test: run-pass
#$ rules: ECS-6, ECS-3
#$ profiles: debug, development
#$ stdout: [0, 1, 2, 3, 4, 5]
#$ [0, 5, 2, 4, 6]
#$ [0, 5, 2, 4, 6]
#$ [0, 6, 2]
# `[ECS-6]` — "Iteration order is insertion order with swap-remove holes: deterministic for a
# given sequence of operations." A query walks its driving storage in dense order: entities in
# the order their component was added, and a removal moves the last entity into the hole. Removing
# `P` from 1 and then 3 gives 0 5 2 4; 6 is added at the end. A second world given the same
# operations iterates the same way. Destroying 5 and 4 removes them from every storage alike.
from std.ecs import World, Query, Component, Entity

@derive(Copy, Component)
struct P:
    pass

fn order(mut q: Query[(Entity, P)], unused: ()):
    seen: Array[u32] = []
    for e, _ in q.iter():
        seen.push(e.index())
    println(seen)

fn build(show: bool) -> (World, Array[Entity]):
    w = World()
    es: Array[Entity] = []
    for i in 0..7:
        es.push(w.create())
    for i in 0..6:
        w.add(es[i], P())
    if show:
        w.run(order, ())
    w.remove[P](es[1])
    w.remove[P](es[3])
    w.add(es[6], P())
    return (w, es)

fn main():
    (w, es) = build(true)
    (again, _) = build(false)
    w.run(order, ())
    again.run(order, ())
    w.destroy(es[5])
    w.destroy(es[4])
    w.run(order, ())
