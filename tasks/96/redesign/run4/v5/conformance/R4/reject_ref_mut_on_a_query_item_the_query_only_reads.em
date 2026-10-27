#$ test: compile-fail
#$ rules: R4
#$ help: fn slow(ref mut q: Query[(ref mut Position, ref mut Velocity)], dt: float):
## R4: a query item the query only reads is not given a `ref mut` loop name: the query's type says what
## the system may change (AN-15, E3027; completeness G6).

from std.ecs import World, Query, Component

@derive(Copy, Component)
struct Position:
    x: float

@derive(Copy, Component)
struct Velocity:
    dx: float

fn slow(ref mut q: Query[(ref mut Position, Velocity)], dt: float):
    for ref mut pos, ref mut vel in q:    #$ error[E3027]: the query reads `Velocity` only; to change it, ask for it: `Query[(ref mut Position, ref mut Velocity)]`
        vel.dx *= 0.5
        pos.x += vel.dx * dt

fn main():
    w = World()
    e = w.create()
    w.add(e, Position(x=0.0))
    w.add(e, Velocity(dx=2.0))
    w.run(slow, 1.0)
