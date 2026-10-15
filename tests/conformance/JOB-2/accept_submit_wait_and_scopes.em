#$ test: run-pass
#$ rules: JOB-2, THR-5, THR-11, CLO-7
#$ stdout: 55
#$ [10, 20, 30] [8, 10, 12] hello
# `[JOB-2]` — `jobs.submit(owned f)` runs `f` as a job on the pool and gives a `JobHandle`, which
# `jobs.wait` waits on; a job takes over what it captures (`owned fn`). `jobs.scope()`'s jobs may
# borrow the block's places, each borrow lasting until the block ends, which waits for them.

import std.jobs
from std.sync import Atomic

static TOTAL: Atomic[int] = Atomic(0)

fn scale(mut xs: MutSpan[int], k: int):
    for i in 0..xs.len():
        xs[i] *= k

fn main():
    handles: Array[jobs.JobHandle] = []
    for i in range(1, 11):
        n = i
        handles.push(jobs.submit(owned fn() => TOTAL.fetch_add(n)))
    for h in handles:
        jobs.wait(h)
    println(TOTAL.load())
    left = [1, 2, 3]
    right = [4, 5, 6]
    greeting = "hello".to_string()
    with s = jobs.scope():
        s.submit(fn() => scale(left, 10))
        s.submit(fn() => scale(right, 2))
    jobs.wait(jobs.submit(owned fn() => println(left, right, greeting)))
