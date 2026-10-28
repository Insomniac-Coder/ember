#$ test: run-pass
#$ rules: R8, R4
#$ stdout: 2.0 1.0
#$ assert-c-count: contains("name the same element of") == 0
## R8 (f): inside `if i != j:` the two `ref mut` names are proved apart, so nothing is compared while
## running, and the equal pair is skipped. The C holds no compare: its stop text (section 2) is not in
## it; the two `run_` tests of the compare pin that the same words are there when it runs.

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
