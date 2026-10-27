#$ test: run-pass
#$ rules: R15, R8
#$ stdout: 2
## R15: "A `spawn` inside a loop over what the thread changes under a lock the loop holds is allowed,
## the thread waiting for the lock" (verdict 52; AN-76). The loop name `r` is given to the thread with
## a word, as an unscoped thread's captures need (R15, E7004); the handles are joined after the block.

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
            handles.push(thread.spawn(fn() with copy of r changes RESULTS: worker(r)))
    for h in owns handles:
        h.join()
    with RESULTS_LOCK:
        println(RESULTS.len())
