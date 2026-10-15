#$ test: compile-fail
#$ rules: JOB-2, THR-5
#$ error[E3014]: a `JobScope` may not be moved
# `[JOB-2]` — the `JobScope` binding stays where its `with` put it, as `[THR-5]` says of a `Scope`.

import std.jobs

fn main():
    with s = jobs.scope():
        _other = s
