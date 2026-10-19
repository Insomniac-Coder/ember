#$ test: run-pass
#$ rules: ECS-5, ECS-3
#$ profiles: debug, development
#$ stdout: saw 4 queued 3
#$ alive 3 [0, 2] [2] [0, 3, 2]
#$ saw 3 queued 1
#$ [0, 3, 2] []
# `[ECS-5]` — "Adding, removing and destroying during iteration go through a `Commands` buffer
# (`q.commands().destroy(e)`) that is applied after the system returns": nothing changes under
# the query while it runs (the first system sees all four entities though it destroys one and
# adds and removes components), and every change is made, in order, once it has returned. A
# system that iterates with `iter_mut` takes the buffer before the loop: the loop borrows the
# query mutably.
from std.ecs import World, Query, Mut, Component, Entity

@derive(Copy, Component)
struct Hp:
    n: int

@derive(Copy, Component)
struct Dead:
    pass

@derive(Copy, Component)
struct Shield:
    pass

fn judge(mut q: Query[(Entity, Hp)], unused: ()):
    saw = 0
    for e, hp in q.iter():
        saw += 1
        if hp.n <= 0:
            q.commands().destroy(e)
        elif hp.n < 5:
            q.commands().add(e, Dead())
        elif hp.n > 50:
            q.commands().remove[Shield](e)
    println("saw", saw, "queued", q.commands().len())

fn heal(mut q: Query[(Entity, Mut[Hp], Option[Dead])], amount: int):
    cmds = q.commands()
    saw = 0
    for e, hp, dead in q.iter_mut():
        saw += 1
        hp.n += amount
        if dead.is_some():
            cmds.remove[Dead](e)
    println("saw", saw, "queued", cmds.len())

fn indices(es: Array[Entity]) -> Array[u32]:
    out: Array[u32] = []
    for e in es:
        out.push(e.index())
    return out

fn main():
    w = World()
    for n in [10, 0, 3, 99]:
        e = w.create()
        w.add(e, Hp(n))
        w.add(e, Shield())
    w.run(judge, ())
    println("alive", w.len(), indices(w.entities_with[Shield]()), indices(w.entities_with[Dead]()), indices(w.entities_with[Hp]()))
    w.run(heal, 1)
    println(indices(w.entities_with[Hp]()), indices(w.entities_with[Dead]()))
