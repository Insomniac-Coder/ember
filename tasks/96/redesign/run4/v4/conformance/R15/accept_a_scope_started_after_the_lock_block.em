#$ test: run-pass
#$ rules: R15
#$ stdout: 0
#$ stdout: 1
## E7018's way out: the scope starts after the `with RESULTS_LOCK:` block, so its task takes the lock
## once this thread has let it go, and the scope's end waits for nothing this thread holds.

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
        println(RESULTS.len())
    with scope = thread.scope():
        scope.spawn(fn() changes RESULTS: worker(2))
    with RESULTS_LOCK:
        println(RESULTS.len())
