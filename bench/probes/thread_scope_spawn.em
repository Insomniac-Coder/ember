# Open a scope and run one task in it 20 thousand times, the task reading a local the scope borrows
# (`[THR-5]`, `[THR-11]`): the cost of a scope and its spawn and join. The C twin starts and joins
# the platform's threads the same way, with the result in a heap block (as `thread_spawn_join.c`).

import std.thread

fn step(values: Span[int], i: int) -> int:
    return values[i % 4] % 7

fn main():
    values = [3, 5, 8, 13]
    total = 0
    for i in range(20_000):
        with scope = thread.scope():
            h = scope.spawn(fn() => step(values, i))
            total += h.join()
    println(total)
