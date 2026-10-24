# W1's program in the combined model's proposed syntax (phase3b-combined.md 6.1), against today's
# std.ecs names where they exist. Checked with today's `ember check` only to record what it says.
from std.ecs import World

struct Position:
    x: f32
    y: f32

struct Velocity:
    dx: f32
    dy: f32

fn main():
    reg = World()
    for i in 0..1_000_000:
        e = reg.create()
        reg.add(e, Position(x=(i % 1000) as f32, y=(i // 1000) as f32))
        if i % 2 == 0:
            reg.add(e, Velocity(dx=(i % 7) as f32 * 0.5, dy=(i % 5) as f32 * 0.25))
    for s in 0..100:
        for e, pos, vel in reg.group[(Position, Velocity)]():
            pos.x += vel.dx
            pos.y += vel.dy
            if e.id % 100 == s:
                reg.destroy(e)
                n = reg.create()
                reg.add(n, Position(x=0.0, y=0.0))
                reg.add(n, Velocity(dx=1.0, dy=1.0))
