#$ test: run-pass
#$ rules: ARN-11, DRV-1
#$ profiles: debug, release
#$ stdout: 3 true true
# `[ARN-11]` — `@derive(Zeroable)` on an enum is proved: zero bytes are its variant whose
# discriminant is zero, here `Idle` with its zero `count`, so `alloc_zeroed` may make them.

@derive(Zeroable)
enum Job:
    Idle(count: int)
    Busy(left: int, of: int)

fn main():
    frame = Arena.with_capacity(256)
    jobs = frame.alloc_zeroed[Job](3)
    first = match jobs[0]:
        Job.Idle(count) => count == 0
        Job.Busy(..) => false
    last = match jobs[2]:
        Job.Idle(count) => count == 0
        Job.Busy(..) => false
    println(jobs.len(), first, last)
