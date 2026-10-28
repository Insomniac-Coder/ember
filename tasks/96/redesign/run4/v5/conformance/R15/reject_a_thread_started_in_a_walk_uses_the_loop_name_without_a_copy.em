#$ test: compile-fail
#$ rules: THR-17, THR-10
## design rules: R15
## R15 (verdicts 65 and 93): "The loop's name used with no word is refused: ... E7004 for a
## `thread.spawn` thread, whose message gives `copy of t`" (verdict 52's line as first written; section
## 3's E7004 loop message, with the line to write). The nearest allowed program is
## `accept_a_thread_started_in_a_walk_under_the_lock_it_needs.em`.

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
            handles.push(thread.spawn(fn(): worker(r)))    #$ error[E7004]: the thread may outlive the loop, and `r` (line 23) names another element each round; give it its own copy: write `handles.push(thread.spawn(fn() copy of r: worker(r)))` in place of line 24
    for h in owns handles:
        h.join()
