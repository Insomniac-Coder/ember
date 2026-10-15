# Copy a `SyncShared[int]` into an array slot and back out, 50 million times: each copy is an atomic
# retain and each overwritten copy an atomic release (`[HEAP-10]`). The C twin makes the same
# atomic count changes on a counted block.

from std.sync import SyncShared

fn main():
    first = SyncShared(1)
    second = SyncShared(2)
    slots = [first, second]
    total = 0
    n = 50_000_000
    for i in range(n):
        slots[i % 2] = slots[(i + 1) % 2]
        total += slots[0].get() + i % 3
    println(total)
