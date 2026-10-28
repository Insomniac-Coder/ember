#$ test: run-pass
#$ rules: CLO-1, FN-4, THR-1
#$ stdout: 2
## design rules: R12, R5, R15
## Verdicts 29, 30 and 86, R12 (rules.md lines 692-702, 726-729), R15 (lines 835-842): each thread's
## function holds a `thread_shared` owner of the log and pushes under its lock; `thread.spawn` takes a
## `thread_shared fn()` (verdict 94), and starting a thread counts as making what its function changes,
## worked out from its body, so neither `start` nor `main` writes anything. AN-75's NEW program, with the
## count of lines read under the lock after both joins.

import std.thread
from std.sync import Lock

class Log:
    lock: Lock = Lock()
    lines: Array[String] guarded by lock = []

    fn push(s: StrView):
        with this.lock:
            this.lines.push(String(s))

fn start(log: thread_shared Log):
    a = thread.spawn(fn() thread_shared log: log.push("a"))
    b = thread.spawn(fn() thread_shared log: log.push("b"))
    a.join()
    b.join()
    with log.lock:
        println(log.lines.len())

fn main():
    start(thread_shared Log())
