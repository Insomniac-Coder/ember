#$ test: run-fail
#$ rules: JOB-5
#$ panics: `jobs.local_arena()` is called outside a job
## `[JOB-5]` — the arena is the running job's: asked for outside any job, it panics.

import std.jobs

fn main():
    arena = jobs.local_arena()
    println(arena.alloc(1) + 0)
