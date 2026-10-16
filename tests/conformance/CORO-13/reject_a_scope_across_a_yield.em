#$ test: compile-fail
#$ rules: CORO-13, THR-6
#$ profiles: debug
# `[CORO-13]` — a `yield` while a `@must_drop` value is live is `E2231`: a suspended generator
# can be forgotten, and the scope's join would never run.

import std.thread

gen fn spawning(xs: Span[int]) -> Generator[int]:
    with s = thread.scope():
        s.spawn(fn() => println(xs.len()))
        yield 1    #$ error[E2231]: is live across this `yield`

fn main():
    data = [1, 2]
    for x in spawning(data):
        println(x)
