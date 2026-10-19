#$ test: run-pass
#$ rules: ECS-3, TYP-35
#$ profiles: debug, development
#$ stdout: a 1 tag 7
#$ c 3 no tag
#$ b 2
# `[ECS-3]` — `Query[(A, Mut[B], Option[C], Not[D])]`, the rule's own shape: each parameter is an
# ordinary type the library maps to its item, `ref A`, `ref mut B` (from `iter_mut`; `ref B` from
# `iter`), `Option[ref C]` and nothing (`()`) for `Not[D]`. An entity matches where it has an `A`
# and a `B` and no `D`; `Option[C]` matches every entity. `Mut` and `Not` are marker structs whose
# parameter is phantom (`[TYP-35]`).
from std.ecs import World, Query, Mut, Not, Component, Entity

@derive(Copy, Component)
struct Name:
    text: str

@derive(Copy, Component)
struct Count:
    n: int

@derive(Copy, Component)
struct Tag:
    n: int

@derive(Copy, Component)
struct Frozen:
    pass

## `Not[T]`'s item is `()`.
fn yields_nothing(item: ()):
    pass

fn bump(mut q: Query[(Name, Mut[Count], Option[Tag], Not[Frozen])], by: int):
    for name, count, tag, _ in q.iter():
        match tag:
            Some(t):
                println(name.text, count.n, "tag", t.n)
            None:
                println(name.text, count.n, "no tag")
    for _, count, _, nothing in q.iter_mut():
        count.n += by
        yields_nothing(nothing)

fn show(mut q: Query[(Name, Count, Not[Frozen])], unused: ()):
    for name, count, _ in q.iter():
        println(name.text, count.n)

fn main():
    w = World()
    a = w.create()
    b = w.create()
    c = w.create()
    d = w.create()
    for e, text, n in [(a, "a", 1), (b, "b", 2), (c, "c", 3)]:
        w.add(e, Name(text))
        w.add(e, Count(n))
    w.add(d, Name("d"))
    w.add(a, Tag(7))
    w.add(b, Frozen())
    w.run(bump, 10)
    w.remove[Frozen](b)
    w.add(a, Frozen())
    w.add(c, Frozen())
    w.run(show, ())
