#$ test: run-pass
#$ rules: SIMD-3, SOA-2, DSJ-3
#$ profiles: debug, release, shipping
#$ stdout: [1.5, 2.5, 3.5, 4.5]
# `[SIMD-3]` (c) — distinct `SoA` columns are apart: the loop over `ps.x` and `ps.v` runs in a
# function of its own whose two column pointers are `restrict`, for every C compiler (the milestone
# test reads the C). Two columns of one container never overlap, so this is derived, not assumed.

struct Particle:
    x: f32
    v: f32

@noinline
fn step(mut ps: SoA[Particle], dt: f32):
    xs = ps.x
    vs = ps.v
    for i in 0..xs.len():
        xs[i] += vs[i] * dt

fn main():
    ps = SoA[Particle].with_capacity(4)
    for i in range(4):
        ps.push(Particle(i as f32, 1.0))
    step(ps, 1.5)
    out: Array[f32] = []
    for p in ps:
        out.push(p.x)
    println(out)
