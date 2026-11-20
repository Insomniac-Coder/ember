#$ test: compile-fail
#$ rules: BRW-5, DIA-25
# `[BRW-5]` — `ref mut a[i]` and `ref mut a[j]` as two arguments of one call are `E3022` unless the compiler
# proves `i` and `j` differ; the message names the callee's two parameters and the line to write, and the
# same position twice is "both are `bodies[i]`".

struct Body:
    v: int

fn collide(ref mut a: Body, ref mut b: Body):
    a.v += b.v
    b.v = 0

fn unproved(i: int, j: int):
    bodies = [Body(v=1), Body(v=2)]
    collide(ref mut bodies[i], ref mut bodies[j])    #$ error[E3022]: `collide` changes `a` and `b` as two different bodies, and `bodies[i]` and `bodies[j]` are the same body when `i == j`; write `if i != j:` above this line and indent it under it

fn twice(i: int):
    bodies = [Body(v=1), Body(v=2)]
    collide(ref mut bodies[i], ref mut bodies[i])    #$ error[E3022]: `collide` changes `a` and `b` as two different bodies, but both are `bodies[i]`

fn proved(i: int, j: int):
    bodies = [Body(v=1), Body(v=2)]
    if i != j:
        collide(ref mut bodies[i], ref mut bodies[j])

fn main():
    unproved(0, 1)
    twice(0)
    proved(0, 1)
