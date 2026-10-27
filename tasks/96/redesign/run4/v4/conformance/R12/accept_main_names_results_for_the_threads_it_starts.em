#$ test: run-pass
#$ rules: R12, R5, R15
#$ stdout: 1 1
#$ stdout: 2
## Verdict 30, R12 (rules.md lines 686-694): `thread.spawn` takes `owns f: fn() changes CHANGED -> R` and
## says `changes CHANGED`, so `main` names what its threads change, `RESULTS`, beside what `tick`
## changes. AN-76's NEW program, with what it made printed after the joins.

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
    println(FRAME, LOG.len())
    with RESULTS_LOCK:
        println(RESULTS.len())
