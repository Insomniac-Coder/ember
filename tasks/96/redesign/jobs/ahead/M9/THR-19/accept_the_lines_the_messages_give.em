# Clause THR-19.1: "call `note()` after the `with log.lock:` block: move line 16 below the block";
# "join after the `with RESULTS_LOCK:` block: move line 16 below the block"; "take the second read
# after the first ends"
#$ test: run-pass
#$ rules: THR-19, DIA-25
#$ stdout: 1
#$ stdout: 1
#$ stdout: 2
#$ stdout: 1
#$ stdout: 1
# Each message's fix, written in.

import std.thread
from std.sync import Lock, RwLock

class Log:
    lock: Lock = Lock()
    lines: Array[String] guarded by lock = []

    fn push(s: StrView):
        with this.lock:
            this.lines.push(owns String(s))

static RESULTS_LOCK: Lock = Lock()
static mut RESULTS: Array[int] guarded by RESULTS_LOCK = []

fn worker(x: int):
    with RESULTS_LOCK:
        RESULTS.push(owns x)

fn main():
    log = Log()
    note = fn(): log.push("x")
    with log.lock:
        println(log.lines.len() + 1)
    note()
    with log.lock:
        println(log.lines.len())
    handles: Array[thread.JoinHandle[void]] = []
    with RESULTS_LOCK:
        handles.push(owns thread.spawn(owns fn(): worker(2)))
    for t in owns handles:
        t.join()
    worker(3)
    with RESULTS_LOCK:
        println(RESULTS.len())
    rw = RwLock([1])
    with g = rw.read():
        println(g.len())
    with h = rw.read():
        println(h.len())
