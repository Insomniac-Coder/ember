#$ test: compile-fail
#$ rules: JOB-3, BRW-1
#$ error[E3022]: `left` is already mutably borrowed
# `[JOB-3]` — `submit_after` orders execution; it does not relax the borrow rules, which treat every
# job of a scope as running at once: two jobs may not both write `left`, ordered or not.

import std.jobs

fn bump(mut xs: MutSpan[int]):
    xs[0] += 1

fn main():
    left = [1, 2]
    with s = jobs.scope():
        a = s.submit(fn() => bump(left))
        s.submit_after([a], fn() => bump(left))
