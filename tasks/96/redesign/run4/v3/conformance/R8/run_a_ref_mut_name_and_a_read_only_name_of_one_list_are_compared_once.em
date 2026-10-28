#$ test: run-fail
#$ rules: R8, R4
#$ panics: `src` and `dst` both name `bodies[1]`
## R8 (f), (g): a `ref mut` name and a read-only name of elements of one list that differ only by an
## index are compared once (section 2: "two names ... of one list's elements, at least one
## changeable"); equal indexes stop the program. The message is not given by the rules (R8-17).

struct Body:
    vel: float

fn copy_vel(ref mut bodies: Array[Body], i: int, j: int):
    src = bodies[i]
    dst = ref mut bodies[j]
    dst.vel = src.vel

fn main():
    bs = [Body(vel=1.0), Body(vel=2.0)]
    copy_vel(ref mut bs, 1, 1)
