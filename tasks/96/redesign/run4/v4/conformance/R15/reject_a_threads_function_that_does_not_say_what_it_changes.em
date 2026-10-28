#$ test: compile-fail
#$ rules: R15, R12
## R12 and R15: a thread's function names what it changes on its first line; "A plain `fn()` changes
## nothing at all" (AN-76's refused `thread.spawn(fn(): worker(1))`).

import std.thread
from std.sync import Lock

static RESULTS_LOCK: Lock = Lock()
static mut RESULTS: Array[int] guarded by RESULTS_LOCK = []

fn worker(x: int) changes RESULTS:
    r = x * x
    with RESULTS_LOCK:
        RESULTS.push(r)

fn main() changes RESULTS:
    a = thread.spawn(fn(): worker(1))    #$ error[E2105]: this function calls `worker`, which changes `RESULTS`; say so on its first line: `fn() changes RESULTS: worker(1)`
    a.join()
