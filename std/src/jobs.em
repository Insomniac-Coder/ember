## `std.jobs` — the job system (Part XI.5).
##
## A pool of worker threads, started by the first job, runs jobs: functions taking nothing and giving
## nothing. `[JOB-1]` — a job's captured state is held in its queue slot when it fits in 64 bytes,
## and in one allocation otherwise (`Alloc`).
##
## `[JOB-2]` — `jobs.submit(owned f)` runs `f` as a job: what it captures moves to the job, must be
## `Send`, and may be no view (a job may outlive every local of the code that made it, `[THR-10]`).
## It gives a `JobHandle`; `jobs.wait(h)` waits for the job to end. `jobs.scope()` gives a
## `JobScope`, which only `with s = jobs.scope():` may hold, and whose end waits for every job
## submitted to it; `s.submit(f)` runs `f` as a job that may borrow what outlives the block, each
## borrow lasting until the block ends, as `[THR-5]` and `[THR-11]` say for a scope's threads.
##
## `[JOB-3]` — `s.submit_after(deps, f)` starts `f` only after the jobs in `deps` have ended. It
## orders the jobs; it does not change the borrow rules, which treat every job of a scope as
## running at once.
##
## The compiler knows these by their names here; the declarations give them their names.

pub struct JobHandle:
    id: u64

pub fn submit(owned f: fn()) -> JobHandle:
    panic("`jobs.submit` is the compiler's")

pub fn wait(h: JobHandle):
    panic("`jobs.wait` is the compiler's")

@must_drop
@view
pub struct JobScope:
    anchor: ref u8
    pending: u32

pub fn scope() -> JobScope:
    panic("`jobs.scope` is the compiler's")

## A job submitted to a scope: what `submit_after` waits on.
@derive(Copy)
@view
pub struct ScopedJob:
    anchor: ref u8
    id: u64
