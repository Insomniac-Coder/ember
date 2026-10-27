#$ test: run-pass
#$ rules: R5, R4
#$ stdout: 2
## R5, verdict 86: a change through a link needs no word, whatever holds the link: `tick` changes the
## clock through the `ref mut` parameter's link, and `main` through its own link (verdicts 63 and 64's
## `changes Clock` is gone with every `changes` clause); the compiler works both out from the bodies.

class Clock:
    frame: int = 0

fn tick(ref mut c: shares mut Clock):
    c.frame += 1

fn main():
    clock = shares mut Clock()
    clock.frame += 1
    tick(ref mut clock)
    println(clock.frame)
