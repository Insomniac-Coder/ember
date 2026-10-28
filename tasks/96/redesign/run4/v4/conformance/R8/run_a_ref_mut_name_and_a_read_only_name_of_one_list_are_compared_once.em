#$ test: run-fail
#$ rules: R8, R4
#$ panics: `src` and `dst` name the same element of `bodies` (index 1); `copy_vel` needs two different bodies
#$ assert-c-count: contains("name the same element of") == 1
## R8 (f), (g): a `ref mut` name and a read-only name of elements of one list that differ only by an
## index are compared once (section 2: "two names ... of one list's elements, at least one
## changeable"); equal indexes stop the program, with section 2's stop text in this program's names.

struct Body:
    vel: float

fn copy_vel(ref mut bodies: Array[Body], i: int, j: int):
    src = bodies[i]
    dst = ref mut bodies[j]
    dst.vel = src.vel

fn main():
    bs = [Body(vel=1.0), Body(vel=2.0)]
    copy_vel(ref mut bs, 1, 1)
