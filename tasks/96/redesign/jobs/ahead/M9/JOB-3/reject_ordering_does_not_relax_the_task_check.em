# Clause JOB-3.1: "`s.submit_after(deps, owns f: once fn())` starts `f` only after the jobs in `deps`
# have finished. It orders execution; it does not relax `[THR-11]`, which treats every job of a scope
# as running at once."
#$ test: compile-fail
#$ rules: JOB-3, THR-11
#$ error[E7007]
# The second job reads what the first changes, after it by `submit_after`: still `E7007`. No text in
# the spec: code only, claimed for the file (Q3).

import std.jobs

fn main():
    xs = [1]
    with s = jobs.scope():
        a = s.submit(owns fn(): xs.push(owns 2))
        s.submit_after([a], owns fn(): println(xs.len()))
    println(xs.len())
