#$ test: compile-fail
#$ rules: R8, R4
#$ help: write `if i != j:` above line 12 and indent lines 12 to 14 under it
## R8 (f), (g) (verdict 98): a read-only name promises that what it reads does not change while it is
## used; `dst` may change `bodies[j]`, which is the body `src` reads when `i == j`, and the indexes are
## not proved different, so the change is refused (E3021).

struct Body:
    vel: float

fn copy_vel(ref mut bodies: Array[Body], i: int, j: int):
    src = bodies[i]
    dst = ref mut bodies[j]
    dst.vel = src.vel    #$ error[E3021]: `dst.vel = src.vel` changes `bodies[j]` through `dst` while `src` reads `bodies[i]`, which is the same body when `i == j`; write `if i != j:` above line 12 and indent lines 12 to 14 under it

fn main():
    bs = [Body(vel=1.0), Body(vel=2.0)]
    copy_vel(ref mut bs, 0, 1)
    println(bs[1].vel)
