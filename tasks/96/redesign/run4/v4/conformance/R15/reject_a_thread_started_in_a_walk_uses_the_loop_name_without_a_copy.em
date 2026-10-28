#$ test: compile-fail
#$ rules: R15
## R15 (verdict 65): "The loop's own variable, which changes every round, ... [is] given to a task or a
## thread with `with copy of`"; an unscoped thread that uses a name with no capture word is E7004
## (verdict 52's line as first written).

import std.thread
from std.sync import Lock

static RESULTS_LOCK: Lock = Lock()
static mut RESULTS: Array[int] guarded by RESULTS_LOCK = []

fn worker(x: int) changes RESULTS:
    r = x * x
    with RESULTS_LOCK:
        RESULTS.push(r)

fn main() changes RESULTS:
    worker(1)
    handles: Array[thread.JoinHandle[void]] = []
    with RESULTS_LOCK:
        for r in RESULTS:
            handles.push(thread.spawn(fn() changes RESULTS: worker(r)))    #$ error[E7004]: this thread may still run in a later round of the loop, when `r` (line 22) names the next element; say what the thread holds: `fn() with copy of r changes RESULTS: worker(r)` (its own copy of `r`)
    for h in owns handles:
        h.join()
