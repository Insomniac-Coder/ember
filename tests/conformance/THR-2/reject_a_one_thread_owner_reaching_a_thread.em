#$ test: compile-fail
#$ rules: THR-2, THR-10
# `[THR-2]` — a one-thread value never reaches another thread: its counts are plain memory (`E7015`).

import std.thread

class Plain:
    n: int = 1

fn main():
    a = shares Plain()
    _t1 = thread.spawn(owns owned fn() => a.n)    #$ error[E7015]: captures `a`, a `shares Plain`
    b = shares mut 1
    _t2 = thread.spawn(owns owned fn() => b + 1)    #$ error[E7015]: captures `b`, a `shares mut i64`
