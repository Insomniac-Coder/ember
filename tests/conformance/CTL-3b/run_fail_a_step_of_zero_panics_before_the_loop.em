#$ test: run-fail
#$ rules: CTL-3b, STD-19
#$ profiles: debug, release, shipping
#$ stdout: before
#$ panics: step_by(0): the step must be positive
# `[CTL-3b]` (ODR-089) — a step that is not positive is checked once, before
# the loop, and panics as `Iterator.step_by` does: no turn runs.

fn main():
    k = 0
    println("before")
    for i in (0..3).step_by(k):
        println(i)
