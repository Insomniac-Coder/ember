#$ test: run-fail
#$ rules: HND-1
#$ panics: stale handle
# `[HND-1]` — `pool[h]` with a stale handle panics.

from std.collections import Pool

fn main():
    pool = Pool[int]()
    h = pool.insert(3)
    pool.remove(h)
    println(pool[h])
