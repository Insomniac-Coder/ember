# Clause BRW-5.2: "The same position twice is refused with "both are `bodies[i]`"."
#$ test: compile-fail
#$ rules: BRW-5, BRW-11, BCK-3
# Only the part of the text the spec gives is pinned.

struct Body:
    v: int

fn collide(ref mut a: Body, ref mut b: Body):
    a.v += b.v
    b.v = 0

fn one_index_twice(i: int):
    bodies = [Body(v=1)]
    collide(ref mut bodies[i], ref mut bodies[i])    #$ error[E3022]: both are `bodies[i]`

fn one_constant_twice():
    bodies = [Body(v=1)]
    collide(ref mut bodies[0], ref mut bodies[0])    #$ error[E3022]: both are `bodies[0]`

fn main():
    one_index_twice(0)
    one_constant_twice()
