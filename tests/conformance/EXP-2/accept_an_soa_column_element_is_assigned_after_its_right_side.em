#$ test: run-pass
#$ rules: EXP-2, SOA-2
#$ profiles: debug, development, release
#$ stdout: 2.5 4.0
## `[EXP-2]` (D-762) — an assignment to an element of an `SoA` column evaluates its right side
## first, which may read the column, then the place once: `p.velocity[i] = p.velocity[i] + dt` and
## `p.velocity[0] += dt` read and write the column through one view.

@derive(Copy)
struct Particle:
    velocity: f32

fn step(mut p: SoA[Particle], dt: f32):
    for i in 0..p.len():
        p.velocity[i] = p.velocity[i] + dt
    p.velocity[0] += dt
    p.velocity[1] *= 2.0

fn main():
    ps = SoA[Particle].with_capacity(4)
    ps.push(Particle(1.5))
    ps.push(Particle(1.5))
    step(ps, 0.5)
    println(ps.velocity[0], ps.velocity[1])
