#$ test: run-pass
#$ rules: DIA-7, DIA-10, DIA-25, FN-2
#$ stdout: 1
# `[DIA-25]` — the line shape B4's help gives compiles: `ref mut` on the parameter and at the call.

struct Clock:
    ticks: i32

fn tick(ref mut clock: Clock):
    clock.ticks += 1

fn main():
    c = Clock(0)
    tick(ref mut c)
    println(c.ticks)
