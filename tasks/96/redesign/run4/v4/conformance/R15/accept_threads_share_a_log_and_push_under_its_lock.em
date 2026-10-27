#$ test: run-pass
#$ rules: R15, R12, R5, R3
#$ stdout: 2
## R15 and R12 (verdicts 29, 30, 44): each thread holds one more owner (`with thread_shared log`),
## changes the log only under its lock, and says `changes Log`; `start` and `main` say it too
## (AN-75's NEW, with a line that prints what the threads did).

import std.thread
from std.sync import Lock

class Log:
    lock: Lock = Lock()
    lines: Array[String] guarded by lock = []

    fn push(s: StrView) changes lines:
        with this.lock:
            this.lines.push(String(s))

fn start(log: thread_shared Log) changes Log:
    a = thread.spawn(fn() with thread_shared log changes Log: log.push("a"))
    b = thread.spawn(fn() with thread_shared log changes Log: log.push("b"))
    a.join()
    b.join()
    with log.lock:
        println(log.lines.len())

fn main() changes Log:
    start(thread_shared Log())
