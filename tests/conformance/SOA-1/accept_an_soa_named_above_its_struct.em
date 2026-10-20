#$ test: run-pass
#$ rules: SOA-1, SOA-2
#$ profiles: debug, development, release
#$ stdout: 5.9 2
## `[SOA-1]` (D-761) — an `SoA[T]` named in a signature above `T`'s declaration has `T`'s columns:
## the declaration order of a module's items does not matter.

fn integrate(mut p: SoA[Particle], dt: f32):
    for i in 0..p.len():
        p.velocity[i] += 9.8 * dt

@derive(Copy)
struct Particle:
    velocity: f32
    mass: int

fn main():
    ps = SoA[Particle].with_capacity(4)
    ps.push(Particle(1.0, 2))
    integrate(ps, 0.5)
    println(ps.velocity[0], ps.mass[0])
