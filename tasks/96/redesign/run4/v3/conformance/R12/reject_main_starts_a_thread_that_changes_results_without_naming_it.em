#$ test: compile-fail
#$ rules: R12, R5, R15
#$ help: fn main() changes FRAME, LOG, RESULTS:
## Verdict 30, R12 (rules.md lines 570-578): a call that runs a function on its own counts as making its
## changes, now or later; `main` starts threads whose function changes `RESULTS`, so it names `RESULTS`.
## AN-76 part 6; rules.md section 3's line 821 message, in this program's lines.

import std.thread
from std.sync import Lock

static mut FRAME: int = 0
static mut LOG: Array[String] = []

fn tick() changes FRAME, LOG:
    FRAME += 1
    LOG.push(String("tick"))

static RESULTS_LOCK: Lock = Lock()
static mut RESULTS: Array[int] guarded by RESULTS_LOCK = []

fn worker(x: int) changes RESULTS:
    r = x * x
    with RESULTS_LOCK:
        RESULTS.push(r)

fn main() changes FRAME, LOG:    #$ error[E2105]: `main` starts a thread whose function changes `RESULTS` (`fn() changes RESULTS: worker(1)`, line 28); add it to `main`'s first line: `fn main() changes FRAME, LOG, RESULTS:`
    tick()
    a = thread.spawn(fn() changes RESULTS: worker(1))
    b = thread.spawn(fn() changes RESULTS: worker(2))
    a.join()
    b.join()
