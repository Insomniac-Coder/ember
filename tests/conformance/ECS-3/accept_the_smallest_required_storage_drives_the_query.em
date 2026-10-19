#$ test: run-pass
#$ rules: ECS-3
#$ profiles: debug, development
#$ stdout: [3, 1]
#$ [3, 1]
#$ [0, 1, 2, 3, 4]
#$ [0, 2, 4]
# `[ECS-3]` — "A query iterates the smallest required storage and probes the others": entities
# come in the order of the shortest storage among the components the query requires, whichever
# position it has in the tuple. `B` is on two entities, added in the order 3, 1; `A` on five.
# `Option[B]` and `Not[B]` require nothing of `B`, so those queries are driven by `A`.
from std.ecs import World, Query, Not, Component, Entity

@derive(Copy, Component)
struct A:
    pass

@derive(Copy, Component)
struct B:
    pass

fn ab(mut q: Query[(Entity, A, B)], unused: ()):
    seen: Array[u32] = []
    for e, _, _ in q.iter():
        seen.push(e.index())
    println(seen)

fn ba(mut q: Query[(B, Entity, A)], unused: ()):
    seen: Array[u32] = []
    for _, e, _ in q.iter():
        seen.push(e.index())
    println(seen)

fn a_maybe_b(mut q: Query[(Entity, A, Option[B])], unused: ()):
    seen: Array[u32] = []
    for e, _, _ in q.iter():
        seen.push(e.index())
    println(seen)

fn a_not_b(mut q: Query[(Entity, A, Not[B])], unused: ()):
    seen: Array[u32] = []
    for e, _, _ in q.iter():
        seen.push(e.index())
    println(seen)

fn main():
    w = World()
    es: Array[Entity] = []
    for i in 0..5:
        e = w.create()
        w.add(e, A())
        es.push(e)
    w.add(es[3], B())
    w.add(es[1], B())
    w.run(ab, ())
    w.run(ba, ())
    w.run(a_maybe_b, ())
    w.run(a_not_b, ())
