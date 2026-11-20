# Clause BRW-5.1: "unless the compiler proves `i` and `j` differ: both constants and different, `j`
# from `i + 1..`, or the line inside `if i != j:`. Nothing is compared while the program runs. (...)
# and compiles under `if i != j:` or in `for j in i + 1..n:`."
#$ test: run-pass
#$ rules: BRW-5, BRW-11, BCK-3
#$ stdout: 3 0
#$ stdout: 3 0
#$ stdout: 6
#$ stdout: 3 0
# Each proof, as arguments of one call and as two names. A `const` item is a constant.

const FIRST: int = 0
const SECOND: int = 1

struct Body:
    v: int

fn collide(ref mut a: Body, ref mut b: Body):
    a.v += b.v
    b.v = 0

fn literals():
    bodies = [Body(v=1), Body(v=2)]
    collide(ref mut bodies[0], ref mut bodies[1])
    println(bodies[0].v, bodies[1].v)

fn named_constants():
    bodies = [Body(v=1), Body(v=2)]
    a = ref mut bodies[FIRST]
    b = ref mut bodies[SECOND]
    a.v += b.v
    b.v = 0
    println(bodies[0].v, bodies[1].v)

fn after(i: int):
    bodies = [Body(v=1), Body(v=2), Body(v=3)]
    for j in i + 1..bodies.len():
        collide(ref mut bodies[i], ref mut bodies[j])
    println(bodies[i].v)

fn guarded(i: int, j: int):
    bodies = [Body(v=1), Body(v=2)]
    if i != j:
        collide(ref mut bodies[i], ref mut bodies[j])
    println(bodies[i].v, bodies[j].v)

fn main():
    literals()
    named_constants()
    after(0)
    guarded(1, 0)
