#$ test: compile-fail
#$ rules: ECS-3
# `[ECS-3]` — `iter_mut` requires `mut q`: a query passed borrowed is iterated with `iter`.
from std.ecs import Query, Mut, Component

@derive(Copy, Component)
struct Count:
    n: int

fn reset(q: Query[Mut[Count]]):
    for count in q.iter_mut():  #$ error[E3023]: cannot mutate borrowed parameter `q`
        count.n = 0
