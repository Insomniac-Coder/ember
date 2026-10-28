#$ test: compile-fail
#$ rules: R15, R5, R12
## R15: "Starting a thread counts as making what its function changes"; a `main` that does not say so
## is E2105 (verdict 30; AN-76's refused `main`, in a program that keeps only `RESULTS`).

import std.thread
from std.sync import Lock

static RESULTS_LOCK: Lock = Lock()
static mut RESULTS: Array[int] guarded by RESULTS_LOCK = []

fn worker(x: int) changes RESULTS:
    r = x * x
    with RESULTS_LOCK:
        RESULTS.push(r)

fn main():    #$ error[E2105]: `main` starts a thread whose function changes `RESULTS` (`fn() changes RESULTS: worker(1)`, line 18); add it to `main`'s first line: `fn main() changes RESULTS:`
    a = thread.spawn(fn() changes RESULTS: worker(1))
    a.join()
