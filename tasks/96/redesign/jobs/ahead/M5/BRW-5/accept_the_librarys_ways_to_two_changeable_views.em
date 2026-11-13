# Clause BRW-5.3: "`get_pair_mut(i, j)`, `split_at`, `chunks_mut`, `for ref mut x in xs:`, `swap(i,
# j)` and `SoA` columns are the library's ways to hold two changeable views of one container."
#$ test: run-pass
#$ rules: BRW-5, SPN-3, SPN-4, SPN-5, SOA-2, STD-15
#$ stdout: [3, 2, 3]
#$ stdout: [13, 12, 3]
#$ stdout: [14, 12, 4]
#$ stdout: [15, 13, 5]
#$ stdout: [13, 15, 5]
#$ stdout: 1.0
# Each way changes two elements of one list (or two columns of one `SoA`) with no index proof.

struct Particle:
    pos: float
    vel: float

fn step(ref mut pos: MutSpan[float], vel: Span[float]):
    for i in 0..pos.len():
        pos[i] += vel[i]

fn main():
    xs = [1, 2, 3]
    if xs.get_pair_mut(0, 1) exists as (ref mut a, ref b):
        a += b
    println(xs)
    left, right = ref mut xs.as_mut_span().split_at(1)
    left[0] += 10
    right[0] += 10
    println(xs)
    for ref mut c in xs.chunks_mut(2):
        c[0] += 1
    println(xs)
    for ref mut x in xs:
        x += 1
    println(xs)
    xs.swap(0, 1)
    println(xs)
    ps = SoA[Particle].with_capacity(1)
    ps.push(owns Particle(pos=0.0, vel=1.0))
    step(ref mut ps.pos, ps.vel)
    println(ps[0].pos)
