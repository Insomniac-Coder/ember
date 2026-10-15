#$ test: compile-fail
#$ rules: THR-5
#$ error[E3014]: `thread.scope()` must be bound by `with`
# `[THR-5]` — a `Scope` lives exactly as long as a `with` block.

import std.thread

fn main():
    s = thread.scope()
