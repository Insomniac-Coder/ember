#$ test: run-pass
#$ rules: R15, R5, R12
#$ stdout: 1 1 2
## R15: "Starting a thread counts as making what its function changes" (R5, R12), worked out from the
## program: `main` changes `FRAME` and `LOG` itself and `RESULTS` through its threads, and its first line
## says none of it (verdict 86; AN-76's NEW, with a line that prints what the threads did). Renamed
## from step 3's `accept_main_names_what_its_threads_change.em`.

import std.thread
from std.sync import Lock

static mut FRAME: int = 0
static mut LOG: Array[String] = []

fn tick():
    FRAME += 1
    LOG.push(String("tick"))

static RESULTS_LOCK: Lock = Lock()
static mut RESULTS: Array[int] guarded by RESULTS_LOCK = []

fn worker(x: int):
    r = x * x
    with RESULTS_LOCK:
        RESULTS.push(r)

fn main():
    tick()
    a = thread.spawn(fn(): worker(1))
    b = thread.spawn(fn(): worker(2))
    a.join()
    b.join()
    with RESULTS_LOCK:
        println(FRAME, LOG.len(), RESULTS.len())
