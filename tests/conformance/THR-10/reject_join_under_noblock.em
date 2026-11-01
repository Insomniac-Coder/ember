#$ test: compile-fail
#$ rules: THR-10, EFF-5
#$ error[E4003]: `@noblock` function `finish` reaches an operation that may wait
# `[THR-10]` — `join` waits for the thread, so `@noblock` refuses it.

import std.thread

@noblock
fn finish(owns h: thread.JoinHandle[int]) -> int:
    return h.join()

fn main():
    println(finish(owns thread.spawn(owns owned fn() => 1)))
