#$ test: compile-fail
#$ rules: THR-3
#$ error[E2020]: `Once()` takes no arguments
# `[THR-3]` — a `Once` starts not run; it is built from nothing.

from std.sync import Once

fn main():
    o = Once(1)
