#$ test: run-pass
#$ rules: HND-3, HND-2
#$ profiles: development
#$ stdout: 4095 1 false
#$ stdout: 1048576 true
# `[HND-3]` — a slot whose generation is exhausted is retired: with `Split20`'s 12-bit generations
# a slot serves 4096 values, then a new slot is used and the old handles stay stale. When every
# slot is live (or retired), `try_insert` returns `Err(CapacityError.Full)`.

from std.collections import Pool, Split20, CapacityError

fn main():
    pool = Pool[int, Split20]()
    first = pool.insert(0)
    last = first
    for value in 1..4096:
        pool.remove(last)
        last = pool.insert(value)
    pool.remove(last)
    next = pool.insert(-1)
    println(last.generation(), next.index(), pool.get(first).is_some())
    full = Pool[int, Split20]()
    count = 0
    while true:
        match full.try_insert(count):
            Ok(_):
                count += 1
            Err(CapacityError.Full):
                break
    println(count, full.len() == count)
