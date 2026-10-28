#$ test: compile-fail
#$ rules: R4, R3
#$ help: fn drift(ref mut q: Query[(ref mut Position, Velocity)], dt: float):
## R4, rules section 3's removed words: `Mut[T]` in a query is gone; a query item the system changes is
## `ref mut T` (AN-15, E1010; completeness G8).

from std.ecs import World, Query, Component

@derive(Copy, Component)
struct Position:
    x: float

@derive(Copy, Component)
struct Velocity:
    dx: float

fn drift(ref mut q: Query[(Mut[Position], Velocity)], dt: float):    #$ error[E1010]: `Mut[Position]` is now `ref mut Position`: write `Query[(ref mut Position, Velocity)]`
    for ref mut pos, ref vel in q:
        pos.x += vel.dx * dt

fn main():
    w = World()
    e = w.create()
    w.add(e, Position(x=0.0))
    w.add(e, Velocity(dx=2.0))
    w.run(drift, 1.0)
