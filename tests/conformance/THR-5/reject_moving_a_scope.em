#$ test: compile-fail
#$ rules: THR-5
#$ error[E3014]: a `Scope` may not be moved
# `[THR-5]` — the `Scope` binding stays where its `with` put it.

import std.thread

fn main():
    with scope = thread.scope():
        _other = scope
