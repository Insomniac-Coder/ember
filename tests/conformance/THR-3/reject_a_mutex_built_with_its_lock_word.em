#$ test: compile-fail
#$ rules: THR-3
#$ error[E2020]: `Mutex(value)` takes one argument, the value it holds
# `[THR-3]` — a `Mutex` is built from its value; its lock is the runtime's.

from std.sync import Mutex

fn main():
    m = Mutex(5, 1)
