#$ test: run-pass
#$ rules: JOB-3, JOB-2
#$ stdout: [10, 20, 30] [8, 10, 12] 66
# `[JOB-3]` — `s.submit_after(deps, f)` starts `f` only after the jobs in `deps` have ended: the
# sum sees both scaled lists.

import std.jobs
from std.sync import Atomic

static SUM: Atomic[int] = Atomic(0)

fn scale(mut xs: MutSpan[int], k: int):
    for i in 0..xs.len():
        xs[i] *= k

fn add(xs: Span[int], ys: Span[int]):
    total = 0
    for x in xs:
        total += x
    for y in ys:
        total += y
    SUM.store(total)

fn main():
    left = [1, 2, 3]
    right = [4, 5, 6]
    with s = jobs.scope():
        s.submit(fn() => scale(left, 10))
        s.submit(fn() => scale(right, 2))
    with t = jobs.scope():
        first = t.submit(fn() => SUM.store(0))
        t.submit_after([first], fn() => add(left, right))
    println(left, right, SUM.load() - 24)
