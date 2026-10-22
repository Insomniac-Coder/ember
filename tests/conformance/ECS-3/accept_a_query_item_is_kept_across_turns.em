#$ test: run-pass
#$ rules: ECS-3, LT-1a, CTL-1
#$ profiles: debug, development
#$ stdout: 9
#$ 5 9
# `[ECS-3]` with `[LT-1a]` (ODR-211) — an item of `q.iter()` borrows the query's columns, not the
# iterator (`QueryIter.next` declares `@borrows(self.view)`), so the "best so far" idiom keeps one
# across turns and after the loop, and two items are held at once.
from std.ecs import World, Query, Component

@derive(Copy, Component)
struct Health:
    hp: int

fn strongest(mut q: Query[Health], unused: ()):
    best: Option[ref Health] = None
    for h in q.iter():
        match best:
            Some(b):
                if h.hp > b.hp:
                    best = Some(h)
            None:
                best = Some(h)
    println(best.unwrap().hp)

fn first_two(mut q: Query[Health], unused: ()):
    it = q.iter()
    a = it.next()
    b = it.next()
    println(a.unwrap().hp, b.unwrap().hp)

fn main():
    world = World()
    for hp in [5, 9, 2]:
        e = world.create()
        world.add(e, Health(hp))
    world.run(strongest, ())
    world.run(first_two, ())
