#$ test: compile-fail
#$ rules: R15
## E7018: "a join of a thread (a scope's end included) whose function changes it", inside the lock's
## `with` block (section 4). The rules give the message for `join` only; this one is written to R18.

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
        with scope = thread.scope():    #$ error[E7018]: the `with scope` block (line 19) waits at its end for the task started on line 20, whose function changes `RESULTS` under `RESULTS_LOCK`, and this thread holds that lock, so each would wait for the other for ever; write the `with scope = thread.scope():` block after the `with RESULTS_LOCK:` block
            scope.spawn(fn() changes RESULTS: worker(2))
