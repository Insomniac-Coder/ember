#$ test: run-fail
#$ rules: HND-3
#$ profiles: development
#$ panics: every slot is retired or live
# `[HND-3]` — `insert` into a pool whose every slot is live panics.

from std.collections import Pool, Split20

fn main():
    pool = Pool[int, Split20]()
    for i in 0..1048577:
        pool.insert(i)
