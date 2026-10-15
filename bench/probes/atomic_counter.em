# Add to an `Atomic[int]` 100 million times with `fetch_add`, which checks the result as `+` does
# (`[THR-14]`): the cost of the atomic and its check. The C twin makes the same atomic add and the
# same check.

from std.sync import Atomic, MemoryOrder

fn main():
    hits = Atomic(0)
    n = 100_000_000
    for i in range(n):
        hits.fetch_add(i % 3, MemoryOrder.Relaxed)
    println(hits.load())
