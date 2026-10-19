#$ test: run-pass
#$ rules: SOA-2, SOA-1, SOA-6, BRW-5, BRW-4
#$ profiles: debug, development
#$ stdout: 1 1.984 0.016
#$ 1.984 1.0
# Part XII.1's example: `integrate(ps.pos, ps.vel, 0.016)` borrows two columns at once, the first
# mutably (`[SOA-2]`: columns are disjoint places, `[BRW-5]`); a column converts to a `Span` where
# one is expected, as an `Array` place does; `ps[0].life -= …` writes one element of one column
# (`[SOA-6]`); `get` gathers a copy. (`integrate` takes `mut pos`: `[FN-1]`.)

from std.math import Vec3

struct Particle:
    pos: Vec3
    vel: Vec3
    life: f32

fn integrate(mut pos: MutSpan[Vec3], vel: Span[Vec3], dt: f32):
    for i in 0..pos.len():
        pos[i] += vel[i] * dt

fn main():
    ps = SoA[Particle].with_capacity(100_000)
    ps.push(Particle(pos=Vec3.ZERO, vel=Vec3(1.0, 0.0, 0.0), life=2.0))
    integrate(ps.pos, ps.vel, 0.016)
    ps[0].life -= 0.016
    p = ps.get(0)
    println(ps.len(), ps[0].life, ps[0].pos.x)
    match p:
        Some(q):
            println(q.life, q.vel.x)
        None:
            println("none")
