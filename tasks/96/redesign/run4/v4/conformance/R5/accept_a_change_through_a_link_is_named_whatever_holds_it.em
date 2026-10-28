#$ test: run-pass
#$ rules: R5, R4
#$ stdout: 2
## R5, verdicts 63, 64: the nearest allowed program: `tick` names `changes Clock` for its change through
## the `ref mut` parameter's link, and `main` for its change through its own link (and for `tick`'s).

class Clock:
    frame: int = 0

fn tick(ref mut c: shares mut Clock) changes Clock:
    c.frame += 1

fn main() changes Clock:
    clock = shares mut Clock()
    clock.frame += 1
    tick(ref mut clock)
    println(clock.frame)
