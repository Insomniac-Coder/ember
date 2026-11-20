# Clause ECS-9.1: "No scheduler. The library runs no systems: a program calls its own functions on the
# registry."
#$ test: run-pass
#$ rules: ECS-9, ECS-4
#$ stdout: 2
# Two "systems" are ordinary functions, called one after the other.

from std.ecs import Registry

@derive(Copy)
struct Hp:
    n: int

fn hurt(ref mut reg: Registry):
    for e, ref mut h in reg.view_mut[Hp]():
        h.n -= 1

fn count(reg: Registry) -> int:
    n = 0
    for e, h in reg.view[Hp]():
        n += h.n
    return n

fn main():
    reg = Registry()
    reg.emplace(reg.create(), owns Hp(n=3))
    hurt(ref mut reg)
    println(count(reg))
