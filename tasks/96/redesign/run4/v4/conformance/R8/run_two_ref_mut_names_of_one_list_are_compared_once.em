#$ test: run-fail
#$ rules: R8, R4
#$ panics: `a` and `b` name the same element of `bodies` (index 1); `swap_vel` needs two different bodies
#$ assert-c-count: contains("name the same element of") == 1
## R8 (f): two `ref mut` names of elements of one list that differ only by an index are compared once,
## as R4 compares two arguments; equal indexes stop the program (section 2, the compare of two
## indexes), with section 2's stop text in this program's names.

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
