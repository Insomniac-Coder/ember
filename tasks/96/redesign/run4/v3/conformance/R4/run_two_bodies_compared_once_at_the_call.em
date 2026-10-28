#$ test: run-pass
#$ rules: R4
#$ stdout: -1.0 1.0
## R4, rules section 2: where two arguments differ only by an index of one list, one compare before the
## call stops the program if they are equal; here they differ and the call runs (AN-03's `step`).

struct Body:
    pos: float
    vel: float

fn collide(ref mut a: Body, ref mut b: Body):
    t = a.vel
    a.vel = b.vel
    b.vel = t

fn step(ref mut bodies: Array[Body], i: int, j: int):
    collide(ref mut bodies[i], ref mut bodies[j])

fn main():
    bs = [Body(pos=0.0, vel=1.0), Body(pos=1.0, vel=-1.0)]
    step(ref mut bs, 0, 1)
    println(bs[0].vel, bs[1].vel)
