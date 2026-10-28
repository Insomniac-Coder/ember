#$ test: run-pass
#$ rules: THR-1, CLO-1, FN-4, OWN-8
#$ stdout: 2
## design rules: R15, R12, R5, R3
## R15 and R12 (verdicts 44, 86 and 94): each thread holds one more owner (`fn() thread_shared log`),
## and changes the log only under its lock; what the threads, `start` and `main` change is worked out,
## and no first line says it (AN-75's NEW, with a line that prints what the threads did).

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
