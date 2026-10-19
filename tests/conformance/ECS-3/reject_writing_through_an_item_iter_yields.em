#$ test: compile-fail
#$ rules: ECS-3
# `[ECS-3]` — `iter` borrows the query shared and yields read-only items, `Mut` parameters
# included: `ref Count`, which nothing writes through.
from std.ecs import Query, Mut, Component

@derive(Copy, Component)
struct Count:
    n: int

fn reset(mut q: Query[Mut[Count]], unused: ()):
    for count in q.iter():
        count.n = 0  #$ error[E3021]: cannot write through a shared reference
