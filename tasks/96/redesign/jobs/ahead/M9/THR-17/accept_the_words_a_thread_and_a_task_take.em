# Clause THR-17.2: "`copy of t` gives it its own copy (...): `thread.spawn(owns fn() copy of r:
# worker(r))`. For a scoped task, `ref t` lets it look at the element `t` is on in that round, with
# no copy"; "A module variable (`RESULTS`) needs no word."
#$ test: run-pass
#$ rules: THR-17, THR-11, THR-21
#$ stdout: 3
#$ stdout: 3
#$ stdout: 2
# A thread takes a copy of the loop's text; a task looks at it with `ref t` and adds to a total (a
# reduction); a module variable reached under its lock needs no word.

import std.thread
from std.sync import Lock

static LENS_LOCK: Lock = Lock()
static mut LENS: Array[int] guarded by LENS_LOCK = []

fn note(n: int):
    with LENS_LOCK:
        LENS.push(owns n)

fn main():
    texts = [String("ab"), String("c")]
    handles: Array[thread.JoinHandle[void]] = []
    for t in texts:
        handles.push(owns thread.spawn(owns fn() copy of t: note(t.len())))
    for h in owns handles:
        h.join()
    total = 0
    with scope = thread.scope():
        for t in texts:
            scope.spawn(owns fn() ref t: total += t.len())
    println(total)
    summed = 0
    with LENS_LOCK:
        for n in LENS:
            summed += n
    println(summed)
    with LENS_LOCK:
        println(LENS.len())
