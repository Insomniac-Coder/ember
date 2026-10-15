# A million particles in an `SoA`, about half kept by `retain` (`[SOA-3]`), then the kept lives
# summed: the cost of moving whole elements (every column) to close the gaps. The C twin compacts
# the same three columns in place.

from std.math import Vec3

struct Particle:
    pos: Vec3
    vel: Vec3
    life: f32

fn main():
    n = 1_000_000
    ps = SoA[Particle].with_capacity(n)
    for i in range(n):
        ps.push(Particle(pos=Vec3(i as f32, 0.0, 0.0), vel=Vec3.ZERO, life=((i * 7) % 10) as f32))
    ps.retain(fn(p) => p.life >= 5.0)
    total = 0.0
    for p in ps:
        total += p.life as f64
    println(ps.len(), (total * 10.0) as int)
