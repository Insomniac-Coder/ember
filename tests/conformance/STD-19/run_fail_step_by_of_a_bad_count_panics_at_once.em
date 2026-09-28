#$ test: run-fail
#$ rules: STD-19, STD-14
#$ profiles: debug, release, shipping
#$ stdout: before
#$ panics: step_by(0): the step must be positive
# `[STD-19]` (ODR-089) — a bad count is the caller's bug and panics when
# `step_by` is called, before any item is read: nothing after it prints.

fn main():
    xs: Array[int] = [1, 2, 3]
    println("before")
    it = xs.iter().step_by(0)
    println("after", it.count())
