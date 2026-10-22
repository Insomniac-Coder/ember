#$ test: run-pass
#$ rules: JOB-5, ARN-6, JOB-2
#$ profiles: debug, development, release
#$ stdout: 68
## `[JOB-5]` — `jobs.local_arena()` is an arena view for the running job: a `ScopedArena` of the
## arena the job system keeps for each worker, rewound when the job ends. A job and the functions it
## calls each take a view and allocate from it, two views live at once; nothing it allocates
## outlives the call that asked (`reject_…`).

import std.jobs
from std.sync import Atomic

static TOTAL: Atomic[int] = Atomic(0)

fn doubled(i: int) -> int:
    arena = jobs.local_arena()
    cell = arena.alloc(i * 2)
    return cell + 0

fn work(n: int):
    arena = jobs.local_arena()
    total = 0
    for i in 0..n:
        kept = arena.alloc(doubled(i))
        total += kept
    TOTAL.fetch_add(total)

fn main():
    handles: Array[jobs.JobHandle] = []
    for n in [3, 4, 5, 6]:
        k = n
        handles.push(jobs.submit(owned fn() => work(k)))
    for h in handles:
        jobs.wait(h)
    println(TOTAL.load())
