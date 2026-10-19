#$ test: compile-fail
#$ rules: ECS-3
# `[ECS-3]` — `iter_mut` yields `ref mut` for `Mut` parameters only: a component the query names
# plainly is read (`ref Count`), even from `iter_mut`.
from std.ecs import Query, Mut, Component

@derive(Copy, Component)
struct Count:
    n: int

@derive(Copy, Component)
struct Total:
    n: int

fn add(mut q: Query[(Mut[Total], Count)], unused: ()):
    for total, count in q.iter_mut():
        total.n += count.n
        count.n = 0  #$ error[E3021]: cannot write through a shared reference
