#$ test: run-pass
#$ rules: ECS-3, SOA-6, ECS-2
#$ profiles: debug, development
#$ stdout: ann 13 1.5
#$ bob 3 0.5
#$ ann 26
#$ bob 6
# `[ECS-3]`, `[SOA-6]` — a component stored as an `SoA` (`@component(layout=soa)`, `[ECS-2]`)
# has no `T` in one place to point at: a query yields its element proxies, `SoAMut[T]` for a
# `Mut` parameter from `iter_mut` and `SoARef[T]` otherwise, whose fields are places in the
# columns. A proxy is passed where one is wanted, and writing through `SoAMut` writes the column.
from std.ecs import World, Query, Mut, Component

@derive(Component)
@component(layout=soa)
struct Score:
    name: String
    points: int

@derive(Copy, Component)
struct Weight:
    w: f64

fn points_of(s: SoARef[Score]) -> int:
    return s.points

fn score(mut q: Query[(Mut[Score], Weight)], bonus: int):
    for s, weight in q.iter_mut():
        if weight.w > 1.0:
            s.points += bonus
    for s, weight in q.iter():
        println(s.name, points_of(s), weight.w)
    for s, _ in q.iter_mut():
        s.points *= 2

fn show(mut q: Query[Score], unused: ()):
    for s in q.iter():
        println(s.name, s.points)

fn main():
    w = World()
    a = w.create()
    b = w.create()
    w.add(a, Score("ann".to_string(), 3))
    w.add(b, Score("bob".to_string(), 3))
    w.add(a, Weight(1.5))
    w.add(b, Weight(0.5))
    w.run(score, 10)
    w.run(show, ())
