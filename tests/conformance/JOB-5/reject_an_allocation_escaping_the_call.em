#$ test: compile-fail
#$ rules: JOB-5, LT-4, TYP-15
## `[JOB-5]` — what a job allocates from `jobs.local_arena()` cannot escape the job: the view borrows
## a hidden local of the call that asked for it, so returning an allocation, or keeping one in a
## place that outlives the call, is refused by the region rules.

import std.jobs

fn leak() -> ref mut int:
    arena = jobs.local_arena()
    return arena.alloc(5)    #$ error[E3060]: the job's arena view does not live long enough
    #$ error[E3061]: arena allocation cannot outlive `arena`

fn main():
    kept: Option[ref mut int] = None
    with s = jobs.scope():
        keep = fn():
            arena = jobs.local_arena()
            kept = Some(arena.alloc(1))    #$ error[E3063]: is stored in a captured variable, which the closure's creator owns
        s.submit(keep)
    println(kept is None)
