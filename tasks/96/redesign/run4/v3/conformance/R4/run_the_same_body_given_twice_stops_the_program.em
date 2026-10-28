#$ test: run-fail
#$ rules: R4
#$ panics: `collide` was given `bodies[1]` as both `a` and `b`; test `if i != j:` before the call to skip it
## R4, rules section 2: the one compare before the call stops the program when the two indexes are
## equal, before `collide` runs with two names for one body (AN-03; rules section 3, E3022's row).

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
    step(ref mut bs, 1, 1)
    println(bs[0].vel)
