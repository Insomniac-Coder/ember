#$ test: run-pass
#$ rules: R8, R4
#$ stdout: 2.0 1.0
## R8 (f): inside `if i != j:` the two `ref mut` names are proved apart, so nothing is compared while
## running, and the equal pair is skipped.

struct Body:
    vel: float

fn swap_vel(ref mut bodies: Array[Body], i: int, j: int):
    if i != j:
        a = ref mut bodies[i]
        b = ref mut bodies[j]
        t = a.vel
        a.vel = b.vel
        b.vel = t

fn main():
    bs = [Body(vel=1.0), Body(vel=2.0)]
    swap_vel(ref mut bs, 0, 1)
    swap_vel(ref mut bs, 1, 1)
    println(bs[0].vel, bs[1].vel)
