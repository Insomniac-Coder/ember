# Clause ECS-3.3: "A view a method gives points into the registry (`[LT-1]`), so the registry is not
# changed through another name while it is used (§VII)."
#$ test: compile-fail
#$ rules: ECS-3, LT-1, BRW-1, BRW-17
# A `ref mut` the registry gave, used after `destroy` (a reshape through `reg` while it lives:
# `E3022` at the reshape, `[BRW-1]`); a read-only one used after `emplace` (`E3060` at its later use,
# `[BRW-17]`). No text in the spec for these programs: code only.

from std.ecs import Registry

@derive(Copy)
struct Hp:
    n: int

fn main():
    reg = Registry()
    e = reg.create()
    h = reg.emplace(e, owns Hp(n=1))
    reg.destroy(reg.create())    #$ error[E3022]
    h.n += 1
    r = reg.get[Hp](e)
    reg.emplace(reg.create(), owns Hp(n=2))
    println(r.n)    #$ error[E3060]
