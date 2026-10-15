# Start a thread and join it 20 thousand times, each adding a little and returning it: the cost of
# `thread.spawn` and `join` (`[THR-10]`). The C twin starts and joins the platform's threads the
# same way (CreateThread on Windows, pthreads elsewhere), with the result in a heap block.

import std.thread

fn step(i: int) -> int:
    return i % 7

fn main():
    total = 0
    for i in range(20_000):
        h = thread.spawn(owned fn() => step(i))
        total += h.join()
    println(total)
