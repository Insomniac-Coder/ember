#$ test: compile-fail
#$ rules: THR-11, BRW-2
#$ error[E3021]: `counts` cannot be written while it is borrowed
# `[THR-11]` — the block may not write what a task reads, joined early or not: the borrow lasts
# until the block ends.

import std.thread

fn read(s: Span[int]) -> int:
    return s[0]

fn main():
    counts = [0, 0]
    with scope = thread.scope():
        h = scope.spawn(fn() => read(counts))
        println(h.join())
        counts[0] = 5
