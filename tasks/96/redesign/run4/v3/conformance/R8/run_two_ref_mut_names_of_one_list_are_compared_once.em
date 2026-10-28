#$ test: run-fail
#$ rules: R8, R4
#$ panics: `a` and `b` both name `bodies[1]`
## R8 (f): two `ref mut` names of elements of one list that differ only by an index are compared once,
## as R4 compares two arguments; equal indexes stop the program (section 2, the compare of two
## indexes). The rules give no message for two names; this one follows R4's (see the matrix, R8-17).

struct Body:
    vel: float

fn swap_vel(ref mut bodies: Array[Body], i: int, j: int):
    a = ref mut bodies[i]
    b = ref mut bodies[j]
    t = a.vel
    a.vel = b.vel
    b.vel = t

fn main():
    bs = [Body(vel=1.0), Body(vel=2.0)]
    swap_vel(ref mut bs, 1, 1)
