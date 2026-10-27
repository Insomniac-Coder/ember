#$ test: run-pass
#$ rules: R4
#$ stdout: 1.0
## R4: inside `if i != j:` the two positions are proved different, so the call needs no compare, and
## `step(ref mut bs, 1, 1)` skips it with nothing stopped.

struct Body:
    pos: float
    vel: float

fn collide(ref mut a: Body, ref mut b: Body):
    t = a.vel
    a.vel = b.vel
    b.vel = t

fn step(ref mut bodies: Array[Body], i: int, j: int):
    if i != j:
        collide(ref mut bodies[i], ref mut bodies[j])

fn main():
    bs = [Body(pos=0.0, vel=1.0), Body(pos=1.0, vel=-1.0)]
    step(ref mut bs, 1, 1)
    println(bs[0].vel)
