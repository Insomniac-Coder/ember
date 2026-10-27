#$ test: run-pass
#$ rules: R16, R4
#$ stdout: 2 1
## R16 and R4 (verdict 98): inside `if i != j:` the compiler proves the two elements differ, so the
## call is allowed in a `@no_alias_checks` function; nothing is compared while running.

struct Body:
    v: int

fn collide(ref mut a: Body, ref mut b: Body):
    t = a.v
    a.v = b.v
    b.v = t

@no_alias_checks
fn step(ref mut bodies: Array[Body], i: int, j: int):
    if i != j:
        collide(ref mut bodies[i], ref mut bodies[j])

fn main():
    bodies = [Body(v=1), Body(v=2)]
    step(ref mut bodies, 0, 1)
    println(bodies[0].v, bodies[1].v)
