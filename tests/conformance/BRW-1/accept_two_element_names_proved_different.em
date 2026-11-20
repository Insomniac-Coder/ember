# Clause BRW-1.5: "Two names of elements of one list, at least one `ref mut`, whose indexes are not
# proved different are refused" -- and those proved different are not: two different constants
# (literals or `const` items), `j` from `i + 1..`, the line inside `if i != j:` (`[BRW-5]`).
#$ test: run-pass
#$ rules: BRW-1, BRW-5, BCK-3
#$ stdout: 3
#$ stdout: 3
#$ stdout: 6
#$ stdout: 3
# Nothing is compared while the program runs: each proof is in the code.

const FIRST: int = 0
const SECOND: int = 1

struct Body:
    x: int

fn literals():
    bodies = [Body(x=1), Body(x=2)]
    a = ref mut bodies[0]
    b = bodies[1]
    a.x += b.x
    println(bodies[0].x)

fn named_constants():
    bodies = [Body(x=1), Body(x=2)]
    a = ref mut bodies[FIRST]
    b = bodies[SECOND]
    a.x += b.x
    println(bodies[0].x)

fn after(i: int):
    bodies = [Body(x=1), Body(x=2), Body(x=3)]
    for j in i + 1..bodies.len():
        a = ref mut bodies[i]
        b = bodies[j]
        a.x += b.x
    println(bodies[i].x)

fn guarded(i: int, j: int):
    bodies = [Body(x=1), Body(x=2)]
    if i != j:
        a = ref mut bodies[i]
        b = bodies[j]
        a.x += b.x
    println(bodies[i].x)

fn main():
    literals()
    named_constants()
    after(0)
    guarded(1, 0)
