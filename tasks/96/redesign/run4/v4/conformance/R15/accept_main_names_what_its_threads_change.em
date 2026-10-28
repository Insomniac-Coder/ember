#$ test: run-pass
#$ rules: R15, R5, R12
#$ stdout: 1 1 2
## R15: "Starting a thread counts as making what its function changes (R5, R12): `main` lists what
## its threads change (`fn main() changes FRAME, LOG, RESULTS:`)" (AN-76's NEW, with a line that
## prints what the threads did).

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

fn main() changes FRAME, LOG, RESULTS:
    tick()
    a = thread.spawn(fn() changes RESULTS: worker(1))
    b = thread.spawn(fn() changes RESULTS: worker(2))
    a.join()
    b.join()
    with RESULTS_LOCK:
        println(FRAME, LOG.len(), RESULTS.len())
