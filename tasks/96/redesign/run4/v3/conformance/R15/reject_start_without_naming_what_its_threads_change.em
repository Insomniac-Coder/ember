#$ test: compile-fail
#$ rules: R15, R5, R12
## R15: starting a thread counts as making what its function changes (verdict 30; AN-75's refused
## `start`).

import std.thread
from std.sync import Lock

class Log:
    lock: Lock = Lock()
    lines: Array[String] guarded by lock = []

    fn push(s: StrView) changes lines:
        with this.lock:
            this.lines.push(String(s))

fn start(log: thread_shared Log):    #$ error[E2105]: `start` starts threads whose functions change `Log` (`fn() with thread_shared log changes Log: log.push("a")`, line 18); add it to `start`'s first line: `fn start(log: thread_shared Log) changes Log:`
    a = thread.spawn(fn() with thread_shared log changes Log: log.push("a"))
    a.join()

fn main() changes Log:
    start(thread_shared Log())
