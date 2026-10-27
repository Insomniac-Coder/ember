#$ test: compile-fail
#$ rules: R15
## R15: "joining it inside that lock is refused (E7018)": each would wait for the other for ever
## (verdict 52; section 3's E7018 message, in this program's lines, naming the chain, R18). The nearest
## allowed program is `accept_a_thread_started_under_the_lock_is_joined_after_it.em`.

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
        c = thread.spawn(fn(): worker(2))
        c.join()    #$ error[E7018]: `c.join()` (line 21) waits for the thread started on line 20, whose function changes `RESULTS` under `RESULTS_LOCK` (through `worker`, line 16), and this thread holds that lock, so each would wait for the other for ever; join after the `with RESULTS_LOCK:` block: move line 21 below the block
