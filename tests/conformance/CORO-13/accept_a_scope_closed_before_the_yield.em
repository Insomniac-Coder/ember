#$ test: run-pass
#$ rules: CORO-13, THR-6
#$ profiles: debug, release
#$ stdout: 2
#$ 1
# `[CORO-13]` — a scope that closes before the next `yield` is unaffected.

import std.thread

gen fn spawning(xs: Span[int]) -> Generator[int]:
    with s = thread.scope():
        s.spawn(fn() => println(xs.len()))
    yield 1

fn main():
    data = [1, 2]
    for x in spawning(data):
        println(x)
