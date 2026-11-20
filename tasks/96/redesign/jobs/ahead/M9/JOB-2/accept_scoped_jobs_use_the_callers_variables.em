# Clause JOB-2.1: "`jobs.scope()` follows `[THR-5]` and `[THR-11]`: `s.submit` on a `JobScope` takes a
# plain function (`owns f: once fn()`), whose jobs may use the caller's own variables, and the
# block's exit joins them"; "`jobs.wait(h)` waits for it"
#$ test: run-pass
#$ rules: JOB-2, THR-11, THR-17
#$ stdout: 3 9
#$ stdout: 5
# §XI.5's two jobs over two spans of the caller's, and an unscoped job given a copy.

import std.jobs
from std.sync import Atomic

static SEEN: Atomic[int] = Atomic(0)

fn double(ref mut xs: MutSpan[int]):
    for i in 0..xs.len():
        xs[i] *= 2

fn add_one(ref mut ys: MutSpan[int]):
    for i in 0..ys.len():
        ys[i] += 1

fn main():
    a = [1]
    b = [8]
    with s = jobs.scope():
        s.submit(owns fn(): add_one(ref mut a.as_mut_span()))
        s.submit(owns fn(): add_one(ref mut b.as_mut_span()))
    with s = jobs.scope():
        s.submit(owns fn(): double(ref mut a.as_mut_span()))
    a[0] -= 1
    println(a[0], b[0])
    n = 5
    h = jobs.submit(owns fn() copy of n: SEEN.fetch_add(n))
    jobs.wait(h)
    println(SEEN.load())
