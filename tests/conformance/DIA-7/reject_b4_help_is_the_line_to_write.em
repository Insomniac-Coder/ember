#$ test: compile-fail
#$ rules: DIA-7, DIA-10, FN-2
#$ error[E3023]: cannot mutate borrowed parameter `clock`
#$ help: write `ref mut clock: Clock` for the parameter, and `ref mut` before the argument at each call (`tick(ref mut …)`)
# `[DIA-7]`, `[DIA-10]`, shape B4 — the help is the line to write (`fn tick(ref mut s: Clock):`,
# `tick(ref mut clock)`), never a category such as "restructure to a single owner".

struct Clock:
    ticks: i32

fn tick(clock: Clock):
    clock.ticks += 1

fn main():
    c = Clock(0)
    tick(c)
