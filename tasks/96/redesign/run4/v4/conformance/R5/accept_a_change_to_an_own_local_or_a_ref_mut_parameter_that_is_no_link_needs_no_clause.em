#$ test: run-pass
#$ rules: R5, R4
#$ stdout: 2
## R5: a change through a `ref mut` parameter or to an own local needs no clause when it is not made
## through a link (verdicts 63, 64 narrow the rule only for links): `clock` is `main`'s one-owner
## object, and `c` names it.

class Clock:
    frame: int = 0

fn tick(ref mut c: Clock):
    c.frame += 1

fn main():
    clock = Clock()
    clock.frame += 1
    tick(ref mut clock)
    println(clock.frame)
