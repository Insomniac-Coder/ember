#$ test: run-pass
#$ rules: JOB-1, JOB-2, EFF-5
#$ stdout: 6 70
# `[JOB-1]` — a job's captured state is held in its queue slot when it fits in 64 bytes, with no
# allocation: a `@noalloc` function may submit such a job. A larger state is one allocation, and
# runs all the same.

import std.jobs
from std.sync import Atomic

static TOTAL: Atomic[int] = Atomic(0)

@noalloc
fn small(a: int, b: int, c: int):
    jobs.wait(jobs.submit(owned fn() => TOTAL.fetch_add(a + b + c)))

fn large():
    values: [int; 10] = [7; 10]
    jobs.wait(jobs.submit(owned fn() => TOTAL.fetch_add(values[0] * 10 - 6)))

fn main():
    small(1, 2, 3)
    print(TOTAL.load(), "")
    large()
    println(TOTAL.load())
