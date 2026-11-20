# Clause ECS-5.1: "while a `for` loop walks a view, adding, removing, creating or destroying through the
# registry is refused when compiling, as for any collection a loop walks (`[BRW-18]`, `E3020`"
#$ test: compile-fail
#$ rules: ECS-5, BRW-18
# The spec's message, with this file's line numbers; adding through the registry is code only.

from std.ecs import Registry

@derive(Copy)
struct Hp:
    n: int

fn main():
    reg = Registry()
    reg.emplace(reg.create(), owns Hp(n=0))
    for e, h in reg.view[Hp]():
        if h.n <= 0:
            reg.destroy(e)    #$ error[E3020]: `reg.destroy(e)` (line 18) changes `reg`, whose pools this loop walks (line 16); collect the entities in a list of your own: write `dead: Array[Entity] = []` before the loop, `dead.push(owns e)` in place of this line, and `for d in dead: reg.destroy(d)` after the loop
    for e, h in reg.view[Hp]():
        reg.emplace(reg.create(), owns Hp(n=h.n))    #$ error[E3020]
