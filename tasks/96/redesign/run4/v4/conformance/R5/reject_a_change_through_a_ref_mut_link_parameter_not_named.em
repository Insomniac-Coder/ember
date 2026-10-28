#$ test: compile-fail
#$ rules: R5, R4
#$ help: fn tick(ref mut c: shares mut Clock) changes Clock:
## R5, verdict 63: a change through a link is named with `changes T` even when the link came in as a
## `ref mut` parameter (E2105).

class Clock:
    frame: int = 0

fn tick(ref mut c: shares mut Clock):
    c.frame += 1    #$ error[E2105]: `tick` changes the `Clock` that `c` links to (`c.frame += 1`, line 11), and a change through a link is named by the type it reaches; write `fn tick(ref mut c: shares mut Clock) changes Clock:`

fn main() changes Clock:
    clock = shares mut Clock()
    clock.frame += 1
    tick(ref mut clock)
    println(clock.frame)
