#$ test: run-pass
#$ rules: CLO-1, FN-4, THR-1
#$ stdout: 1 1
#$ stdout: 2
## design rules: R12, R5, R15
## Verdicts 30, 86 and 94, R12 (rules.md lines 726-729), R15 (lines 835-842): `thread.spawn` takes a
## `thread_shared fn()`, and "starting a thread counts as making what its function changes", worked out
## from its body (`worker` pushes to `RESULTS` under `RESULTS_LOCK`); `main` writes nothing for it, nor
## for what `tick` changes. AN-76's NEW program, with what it made printed after the joins.

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
    println(FRAME, LOG.len())
    with RESULTS_LOCK:
        println(RESULTS.len())
