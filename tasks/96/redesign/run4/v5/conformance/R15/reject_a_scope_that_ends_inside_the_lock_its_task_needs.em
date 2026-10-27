#$ test: compile-fail
#$ rules: R15
## E7018: "a join of a thread (a scope's end included) whose function changes what the lock guards",
## inside that lock's `with` block (section 4). The rules give the message for `join` only; this one is
## written to R18, and names both lines it moves (check-V6's fix: the lock's block keeps a line of its
## own). The nearest allowed program is `accept_a_scope_started_after_the_lock_block.em`.

import std.thread
from std.sync import Lock

static RESULTS_LOCK: Lock = Lock()
static mut RESULTS: Array[int] guarded by RESULTS_LOCK = []

fn worker(x: int):
    r = x * x
    with RESULTS_LOCK:
        RESULTS.push(r)

fn main():
    with RESULTS_LOCK:
        println(RESULTS.len())
        with scope = thread.scope():    #$ error[E7018]: the `with scope` block (line 22) waits at its end for the task started on line 23, whose function changes `RESULTS` under `RESULTS_LOCK` (through `worker`, line 17), and this thread holds that lock (line 20), so each would wait for the other for ever; move lines 22 and 23 below line 21, one level less indented, so that the scope starts after the `with RESULTS_LOCK:` block
            scope.spawn(fn(): worker(2))
