#$ test: run-pass
#$ rules: R4
#$ stdout: -1.0
## R4: where the compiler proves two positions differ (`j` from `i + 1..`), the call runs no compare;
## both parameters are compiled with noalias (AN-03's `all_pairs`).

struct Body:
    pos: float
    vel: float

fn collide(ref mut a: Body, ref mut b: Body):
    t = a.vel
    a.vel = b.vel
    b.vel = t

fn all_pairs(ref mut bodies: Array[Body]):
    for i in 0..bodies.len():
        for j in i + 1..bodies.len():
            collide(ref mut bodies[i], ref mut bodies[j])

fn main():
    bs = [Body(pos=0.0, vel=1.0), Body(pos=1.0, vel=-1.0)]
    all_pairs(ref mut bs)
    println(bs[0].vel)
