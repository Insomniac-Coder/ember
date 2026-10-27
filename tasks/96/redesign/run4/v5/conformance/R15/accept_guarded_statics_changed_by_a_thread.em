#$ test: run-pass
#$ rules: THR-1, FN-4
#$ stdout: 1 1
## design rules: R15, R5
## R15: the lines E7012 gives: both module variables `guarded by` one `static` lock, `tick`'s body
## inside it.

import std.thread
from std.sync import Lock

static TICK_LOCK: Lock = Lock()
static mut FRAME: int guarded by TICK_LOCK = 0
static mut LOG: Array[String] guarded by TICK_LOCK = []

fn tick():
    with TICK_LOCK:
        FRAME += 1
        LOG.push(String("tick"))

fn main():
    a = thread.spawn(fn(): tick())
    a.join()
    with TICK_LOCK:
        println(FRAME, LOG.len())
