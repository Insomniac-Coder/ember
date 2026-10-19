#$ test: run-pass
#$ rules: ECS-3, ECS-4
#$ profiles: debug, development
#$ stdout: Vec3(x=0.016, y=0.0, z=0.0) Vec3(x=1.0, y=0.0, z=0.0)
# Part XII.3's example, as the specification writes it: a system is a function taking a
# `Query[(Mut[Position], Velocity)]`; `iter_mut` yields `ref mut Position` for the `Mut`
# parameter and `ref Velocity` for the other (`[ECS-3]`); `world.run(integrate, 0.016)` borrows
# those two storages and runs it with its one argument (`[ECS-4]`). The system allocates nothing,
# which `@noalloc` checks. `show` then reads both back.
from std.ecs import World, Query, Mut, Component
from std.math import Vec3

@derive(Copy, Component)
struct Position:
    value: Vec3

@derive(Copy, Component)
struct Velocity:
    value: Vec3

@noalloc
fn integrate(mut q: Query[(Mut[Position], Velocity)], dt: f32):
    for pos, vel in q.iter_mut():           # pos: ref mut Position, vel: ref Velocity
        pos.value += vel.value * dt

fn show(mut q: Query[(Position, Velocity)], unused: ()):
    for pos, vel in q.iter():
        println(pos.value, vel.value)

fn main():
    world = World()
    e = world.create()
    world.add(e, Position(Vec3.ZERO))
    world.add(e, Velocity(Vec3(1.0, 0.0, 0.0)))
    world.run(integrate, 0.016)
    world.run(show, ())
