#$ test: run-pass
#$ rules: ECS-4
#$ profiles: debug, development
#$ stdout: a true false
#$ b true true
#$ [Entity(bits=1), Entity(bits=0)] [2, 2]
# `[ECS-4]` — "`world.run(system, args…)` borrows exactly those storages": the storages of the
# components its query names, and no other. This system is given the world itself as its
# argument: it may add a component its query does not name (`Shield`) and read the entities of
# one it does (a read beside the query's own reads), while the query holds `Hp` and `Name`.
from std.ecs import World, Query, Mut, Component, Entity

@derive(Copy, Component)
struct Hp:
    n: int

@derive(Copy, Component)
struct Shield:
    pass

@derive(Copy, Component)
struct Name:
    s: str

fn guard(mut q: Query[(Entity, Mut[Hp], Name)], w: World):
    for e, hp, name in q.iter_mut():
        hp.n += 1
        println(name.s, w.has[Name](e), w.has[Shield](e))
        w.add(e, Shield())

fn report(mut q: Query[Hp], w: World):
    totals: Array[int] = []
    for hp in q.iter():
        totals.push(hp.n)
    println(w.entities_with[Shield](), totals)

fn main():
    w = World()
    for s in ["a", "b"]:
        e = w.create()
        w.add(e, Hp(1))
        w.add(e, Name(s))
    w.add(w.entities_with[Hp]()[1], Shield())
    w.run(guard, w)
    w.run(report, w)
