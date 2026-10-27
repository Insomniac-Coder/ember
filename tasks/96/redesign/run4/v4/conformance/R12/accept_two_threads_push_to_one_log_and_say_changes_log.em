#$ test: run-pass
#$ rules: R12, R5, R15
#$ stdout: 2
## Verdicts 29 and 30, R12 (rules.md lines 605-608, 686-694): each thread's function says `changes Log`;
## starting it counts as making that change, so `start` and `main` name `Log`. AN-75's NEW program, with
## the count of lines read under the lock after both joins.

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
