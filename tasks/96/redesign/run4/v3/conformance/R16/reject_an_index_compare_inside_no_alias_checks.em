#$ test: compile-fail
#$ rules: R16, R4
## R16: "`@no_alias_checks` forbids the run-time alias checks that are left, R4's index compare and
## `assert_disjoint` (E4030)" (AN-81 with AN-03's `step`; section 3's message).

struct Body:
    v: int

fn collide(ref mut a: Body, ref mut b: Body):
    t = a.v
    a.v = b.v
    b.v = t

@no_alias_checks
fn step(ref mut bodies: Array[Body], i: int, j: int):
    collide(ref mut bodies[i], ref mut bodies[j])    #$ error[E4030]: `step` is `@no_alias_checks`, and `collide(ref mut bodies[i], ref mut bodies[j])` (line 16) compares `i` and `j` while running; prove them different (`if i != j:` around the call), or remove `@no_alias_checks`

fn main():
    bodies = [Body(v=1), Body(v=2)]
    step(ref mut bodies, 0, 1)
    println(bodies[0].v, bodies[1].v)
