# Clause ECS-4.1: "`reg.view[A, B]()` (...) gives each item read-only"
#$ test: compile-fail
#$ rules: ECS-4, BRW-15, FN-2a
# `ref mut` on an item of a read-only view (`E3027`, the gap map's rewrite of R4's pair) and a change
# through a plain item (`E3023`). No text in the spec for these: code only.

from std.ecs import Registry

@derive(Copy)
struct Hp:
    n: int

fn main():
    reg = Registry()
    e = reg.create()
    reg.emplace(e, owns Hp(n=1))
    for e2, ref mut h in reg.view[Hp]():    #$ error[E3027]
        h.n += 1
    for e3, h in reg.view[Hp]():
        h.n += 1    #$ error[E3023]
