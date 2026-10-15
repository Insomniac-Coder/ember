#$ test: compile-fail
#$ rules: CT-6
#$ error[E6005]: `x` is a run-time local, which compile time cannot read
#$ help: compute it from constants, or pass the value in at run time
# `[CT-6]` — `comptime(e)` cannot name a run-time local: compile time happens before it exists.

fn main():
    x = 5
    println(comptime(x + 1))
