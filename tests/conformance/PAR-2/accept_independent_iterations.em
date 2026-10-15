#$ test: run-pass
#$ rules: PAR-2, SOA-2
#$ profiles: debug, release
#$ stdout: [2.5, 3.5, 4.5, 5.5]
#$ stdout: [0, 2, 4, 6, 8]
#$ stdout: [1, 3, 3, 0]
# `[PAR-2]` — what keeps the iterations independent: an `SoA` column written at `i`; every access
# to a written `MutSpan` at one offset, `i + 1` here; the body's own names, its own inner loop
# (which may `break`) and `continue` of the loop itself; and whatever it only reads.

struct Particle:
    x: f32
    v: f32

fn step(mut ps: SoA[Particle], dt: f32):
    xs = ps.x
    vs = ps.v
    @parallel
    for i in 0..xs.len():
        xs[i] += vs[i] * dt

fn main():
    ps = SoA[Particle].with_capacity(4)
    for i in range(4):
        ps.push(Particle(i as f32, 1.0))
    step(ps, 2.5)
    out: Array[f32] = []
    for p in ps:
        out.push(p.x)
    println(out)

    values = [0, 1, 2, 3, 4]
    ys = values.as_mut_span()
    @parallel
    for i in 0..4:
        ys[i + 1] = ys[i + 1] * 2
    println(values)

    counts = [0, 0, 0, 0]
    cs = counts.as_mut_span()
    limits = [1, 3, 5, 0]
    @parallel
    for i in 0..4:
        if limits[i] == 0:
            continue
        seen = 0
        for k in 0..10:
            if k == limits[i] or k == 3:
                break
            seen += 1
        cs[i] = seen
    println(counts)
