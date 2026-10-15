# Part XII.1's example at scale: 100 thousand particles in an `SoA`, the position column moved by the
# velocity column 200 times (`[SOA-2]`: two columns borrowed at once), then the positions summed.
# The C twin keeps the same three columns in one 64-byte-aligned block.

from std.math import Vec3

struct Particle:
    pos: Vec3
    vel: Vec3
    life: f32

fn integrate(mut pos: MutSpan[Vec3], vel: Span[Vec3], dt: f32):
    for i in 0..pos.len():
        pos[i] += vel[i] * dt

fn main():
    n = 100_000
    ps = SoA[Particle].with_capacity(n)
    for i in range(n):
        f = (i % 100) as f32
        ps.push(Particle(pos=Vec3.ZERO, vel=Vec3(f, 1.0, 0.5), life=1.0))
    for step in range(200):
        integrate(ps.pos, ps.vel, 0.01)
    total = 0.0
    for i in range(n):
        total += ps[i].pos.x as f64
    println((total * 100.0) as int)
