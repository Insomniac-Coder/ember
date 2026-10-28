#$ test: compile-fail
#$ rules: R15
## R15: "joining it inside that lock is refused (E7018)": each would wait for the other for ever
## (verdict 52; section 3's E7018 message, in this program's lines).

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
        c.join()    #$ error[E7018]: `c.join()` (line 20) waits for the thread started on line 19, whose function changes `RESULTS` under `RESULTS_LOCK`, and this thread holds that lock, so each would wait for the other for ever; join after the `with RESULTS_LOCK:` block
