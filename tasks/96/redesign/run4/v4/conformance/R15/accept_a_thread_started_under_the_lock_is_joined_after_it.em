#$ test: run-pass
#$ rules: R15
#$ stdout: 1
## R15: the line E7018's message gives: the thread started inside the lock is joined after the
## `with RESULTS_LOCK:` block, so it gets the lock once this thread lets it go (verdict 52).

import std.thread
from std.sync import Lock

static RESULTS_LOCK: Lock = Lock()
static mut RESULTS: Array[int] guarded by RESULTS_LOCK = []

fn worker(x: int) changes RESULTS:
    r = x * x
    with RESULTS_LOCK:
        RESULTS.push(r)

fn main() changes RESULTS:
    with RESULTS_LOCK:
        c = thread.spawn(fn() changes RESULTS: worker(2))
    c.join()
    with RESULTS_LOCK:
        println(RESULTS.len())
