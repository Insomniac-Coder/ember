# Clause ECS-5.1: "The values themselves change through the view's `ref mut` items. EnTT allows some
# such changes through paged pools; Ember collects first and changes after."
#$ test: run-pass
#$ rules: ECS-5, DIA-25
#$ stdout: 1
# The message's lines, and a change made through a `ref mut` item.

from std.ecs import Registry, Entity

@derive(Copy)
struct Hp:
    n: int

fn main():
    reg = Registry()
    reg.emplace(reg.create(), owns Hp(n=0))
    reg.emplace(reg.create(), owns Hp(n=2))
    for e, ref mut h in reg.view_mut[Hp]():
        h.n -= 1
    dead: Array[Entity] = []
    for e, h in reg.view[Hp]():
        if h.n <= 0:
            dead.push(owns e)
    for d in dead:
        reg.destroy(d)
    n = 0
    for e, h in reg.view[Hp]():
        n += 1
    println(n)
