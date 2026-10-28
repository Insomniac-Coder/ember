#$ test: compile-fail
#$ rules: R4
#$ help: collide(ref mut bodies[i], ref mut bodies[j])
## R4: two arguments of one call that may both be changed may not name one place; known when
## compiling, it is refused (AN-03, E3022).

struct Body:
    pos: float
    vel: float

fn collide(ref mut a: Body, ref mut b: Body):
    t = a.vel
    a.vel = b.vel
    b.vel = t

fn step(ref mut bodies: Array[Body], i: int, j: int):
    collide(ref mut bodies[i], ref mut bodies[i])    #$ error[E3022]: `collide` changes `a` and `b` as two different bodies, and both are `bodies[i]`; pass two different bodies: `collide(ref mut bodies[i], ref mut bodies[j])`

fn main():
    bs = [Body(pos=0.0, vel=1.0), Body(pos=1.0, vel=-1.0)]
    step(ref mut bs, 0, 1)
    println(bs[0].vel)
