# Clause THR-10.1: "except where the thread makes that change under a lock the walk holds, when the
# new thread waits for the lock until the loop ends"; `[THR-20]`'s program
#$ test: run-pass
#$ rules: THR-10, THR-20, THR-18, THR-17
#$ stdout: 2
# The spec's own program: the threads started in the walk of `RESULTS` push to it under the lock
# the walk holds, and wait for it.

import std.thread
from std.sync import Lock

static RESULTS_LOCK: Lock = Lock()
static mut RESULTS: Array[int] guarded by RESULTS_LOCK = []

fn worker(x: int):
    r = x * x
    with RESULTS_LOCK:
        RESULTS.push(owns r)

fn main():
    worker(1)
    handles: Array[thread.JoinHandle[void]] = []
    with RESULTS_LOCK:
        for r in RESULTS:
            handles.push(owns thread.spawn(owns fn() copy of r: worker(r)))
    for h in owns handles:
        h.join()
    with RESULTS_LOCK:
        println(RESULTS.len())
