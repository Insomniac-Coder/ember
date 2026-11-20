# Clause ECS-4.1: "`reg.view[A, B]()` walks the entities that have every listed component and gives
# each item read-only (...); `reg.view_mut[A, B]()` gives every item `ref mut` (...), which changes
# the values in place"; "`.exclude[C]()` leaves out the entities that have a `C`. A view has
# `each(f)`, `get[T](e)`, `contains(e)`, `size_hint()`, `front()` and `back()`"; "A view takes any
# number of component types, with no limit (`reg.view[Position]()`, `reg.view[Position, Velocity,
# Health, Armor]()`; variadic generics, `[TYP-47]`)"
#$ test: run-pass
#$ rules: ECS-4, TYP-47, ECS-3
#$ stdout: 1.0
#$ stdout: 2
#$ stdout: 1
#$ stdout: 1
#$ stdout: true 2
#$ stdout: 3
# The section's own `integrate` under `@noalloc`, a view of one type, one of four, `exclude`, and
# the view's methods.

from std.ecs import Registry, Entity
from std.math import Vec3

@derive(Copy)
struct Position:
    value: Vec3

@derive(Copy)
struct Velocity:
    value: Vec3

@derive(Copy)
struct Health:
    n: int

@derive(Copy)
struct Armor:
    n: int

@derive(Copy)
struct Frozen:
    on: bool

@noalloc
fn integrate(ref mut reg: Registry, dt: f32):
    for e, ref mut pos, ref mut vel in reg.view_mut[Position, Velocity]():
        pos.value += vel.value * dt

fn count_one(reg: Registry) -> int:
    n = 0
    for e, p in reg.view[Position]():
        n += 1
    return n

fn main():
    reg = Registry()
    a = reg.create()
    reg.emplace(a, owns Position(value=Vec3.ZERO))
    reg.emplace(a, owns Velocity(value=Vec3(1.0, 0.0, 0.0)))
    integrate(ref mut reg, 1.0)
    println(reg.get[Position](a).value.x)
    b = reg.create()
    reg.emplace(b, owns Position(value=Vec3.ZERO))
    reg.emplace(b, owns Frozen(on=true))
    println(count_one(reg))
    reg.emplace(a, owns Health(n=1))
    reg.emplace(a, owns Armor(n=1))
    n = 0
    for e, p, v, h, ar in reg.view[Position, Velocity, Health, Armor]():
        n += h.n + ar.n - 1
    println(n)
    m = 0
    for e, p in reg.view[Position]().exclude[Frozen]():
        m += 1
    println(m)
    v = reg.view[Position]()
    println(v.contains(b), v.size_hint())
    total = 0
    reg.view[Position]().each(owns fn(e, p): total += 1)
    println(total + 1)
