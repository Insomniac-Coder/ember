#$ test: compile-fail
#$ rules: THR-11, BRW-2
#$ error[E3022]: `whole` is already mutably borrowed
# `[THR-11]` — each borrow a task makes lasts until the block ends, so two tasks may not write one
# place.

import std.thread

fn add_to(ref mut s: MutSpan[int], n: int):
    s[0] += n

fn main():
    counts = [0, 0]
    whole = counts.as_mut_span()
    with scope = thread.scope():
        scope.spawn(owns fn() => add_to(ref mut whole, 1))
        scope.spawn(owns fn() => add_to(ref mut whole, 2))
