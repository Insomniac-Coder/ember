#$ perf: soa
## `[TST-28]` — a `SoA` column update: two columns of 100 000 particles, advanced 1 000 times.

struct Particle:
    x: float
    v: float

fn advance(mut xs: MutSpan[float], vs: Span[float]):
    for i in 0..xs.len():
        xs[i] += vs[i]

fn main():
    ps = SoA[Particle].with_capacity(100000)
    for i in 0..100000:
        ps.push(Particle(x=i as float, v=0.5))
    for step in 0..1000:
        advance(ps.x, ps.v)
    total = 0.0
    for x in ps.x:
        total += x
    println(total as int)
