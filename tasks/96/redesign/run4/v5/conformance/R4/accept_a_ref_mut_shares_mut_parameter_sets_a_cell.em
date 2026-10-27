#$ test: run-pass
#$ rules: R4
#$ stdout: 1
## R4, verdict 32: `ref mut s: shares mut Clock` names the caller's place, which holds a `shares mut`
## owner, so `tick` may change the clock; no first line names the change (verdict 86). The nearest
## allowed program to the plain parameter (AN-20).

struct Clock:
    frame: Cell[int]

fn tick(ref mut s: shares mut Clock):
    s.frame.set(s.frame.get() + 1)

fn main():
    clock = shares mut Clock(frame=Cell(0))
    tick(ref mut clock)
    println(clock.frame.get())
