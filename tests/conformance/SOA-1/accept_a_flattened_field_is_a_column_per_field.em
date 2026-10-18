#$ test: run-pass
#$ rules: SOA-1, SOA-2, SOA-6
#$ profiles: debug, release
#$ stdout: 2
#$ 1.0 0.5 3.0
#$ 1.016 0.5 2.984
#$ 1.016 0.5 2.984
#$ Particle(pos=Vec3(x=1.016, y=0.5, z=2.984), vel=Vec3(x=1.0, y=0.0, z=-1.0), life=2.0)
#$ 8.0 0.0
#$ 4.0 6.0
#$ 1
#$ 4.0
# `[SOA-1]` — "A field marked `@soa(flatten)` whose type is a struct is split into one column per
# field of its own (`ps.pos.x`); otherwise a struct field is one column." `pos` is three `f32`
# columns, each a view of its own (`ps.pos.x`), while `vel` stays one column of `Vec3`s. An element
# reads, writes and gathers its flattened field through the columns: `ps[i].pos.x` is a place in
# column `pos.x`, `ps[i].pos` is gathered from three, and writing it whole writes all three.
from std.math import Vec3

@derive(Copy)
struct Particle:
    @soa(flatten)
    pos: Vec3
    vel: Vec3
    life: f32

fn drift(mut xs: MutSpan[f32], vs: Span[Vec3], dt: f32):
    for i in 0..xs.len():
        xs[i] += vs[i].x * dt

# Two columns of one flattened field are two places: both borrowed mutably at once.
fn swap_xy(mut xs: MutSpan[f32], mut ys: MutSpan[f32]):
    for i in 0..xs.len():
        t = xs[i]
        xs[i] = ys[i]
        ys[i] = t

fn main():
    ps = SoA[Particle].with_capacity(4)
    ps.push(Particle(pos=Vec3(1.0, 0.5, 3.0), vel=Vec3(1.0, 0.0, -1.0), life=2.0))
    ps.push(Particle(pos=Vec3(2.0, 0.0, 0.0), vel=Vec3(0.0, 0.0, 0.0), life=1.0))
    println(ps.len())
    println(ps.pos.x[0], ps.pos.y[0], ps.pos.z[0])
    drift(ps.pos.x, ps.vel, 0.016)
    for i in 0..ps.len():
        ps[i].pos.z -= ps[i].vel.x * 0.016
    p = ps[0].pos
    println(p.x, p.y, p.z)
    for q in ps:
        if q.life > 1.5:
            println(q.pos.x, q.pos.y, q.pos.z)
    println(ps.get(0).unwrap())
    ps[1].pos = Vec3(8.0, 0.0, 0.0)
    println(ps.pos.x[1], ps.pos.y[1])
    ps[1] = Particle(pos=Vec3(4.0, 6.0, 1.0), vel=Vec3.ZERO, life=0.5)
    println(ps.pos.x[1], ps.pos.y[1])
    swap_xy(ps.pos.x, ps.pos.y)
    swap_xy(ps.pos.x, ps.pos.y)
    gone = ps.swap_remove(0)
    println(ps.len())
    println(ps.pos.x[0] * gone.life / 2.0)
