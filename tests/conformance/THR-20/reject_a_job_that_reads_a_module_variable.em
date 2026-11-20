#$ test: compile-fail
#$ rules: THR-20, STA-1, JOB-2
# `[THR-20]` — a job submitted outside a scope may run on another thread: a function it runs that only
# reads a module variable of the main thread is `E7012` too.

import std.jobs

static mut FRAME: int = 0

fn report():
    println(FRAME)

fn main():
    FRAME = 3
    jobs.wait(jobs.submit(owns report))    #$ error[E7012]: `report` reads `FRAME`, a module variable of the main thread, and this job runs beside it; guard it with one lock: write `from std.sync import Lock` at the top, `static REPORT_LOCK: Lock = Lock()`, then `static mut FRAME: int guarded by REPORT_LOCK = 0` instead of line 8, and put `report`'s body inside `with REPORT_LOCK:`
