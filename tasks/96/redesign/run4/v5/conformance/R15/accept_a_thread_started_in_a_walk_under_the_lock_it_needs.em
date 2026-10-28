#$ test: run-pass
#$ rules: THR-1, BRW-16
#$ stdout: 2
## design rules: R15, R8
## R15: "A `spawn` inside a loop over what the thread changes under a lock the loop holds is allowed,
## the thread waiting for the lock" (verdict 52; AN-76); the loop's own variable is given with
## `copy of` (verdicts 65 and 83: `thread.spawn(fn() copy of r: worker(r))`); the handles are joined
## after the block.

import std.thread
from std.sync import Lock

static RESULTS_LOCK: Lock = Lock()
static mut RESULTS: Array[int] guarded by RESULTS_LOCK = []

fn worker(x: int):
    r = x * x
    with RESULTS_LOCK:
        RESULTS.push(r)

fn main():
    worker(1)
    handles: Array[thread.JoinHandle[void]] = []
    with RESULTS_LOCK:
        for r in RESULTS:
            handles.push(thread.spawn(fn() copy of r: worker(r)))
    for h in owns handles:
        h.join()
    with RESULTS_LOCK:
        println(RESULTS.len())
