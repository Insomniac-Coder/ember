# Clause ECS-9.1: "The library runs no systems: a program calls its own functions on the registry.
# Work it runs at once goes through `std.thread` or `std.jobs` with the rules of Part XI, and two
# pieces of work run at once that may touch one pool, one changing it, are refused when compiling"
#$ test: compile-fail
#$ rules: ECS-9, THR-11, EFF-24
#$ error[E7007]
# One task changes the `Hp` pool, the other reads it. No text in the spec: code only, claimed for the
# file (M9's Q3).

import std.thread
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
    total = 0
    with scope = thread.scope():
        scope.spawn(owns fn(): hurt(ref mut reg))
        scope.spawn(owns fn(): total += count(reg))
    println(total)
