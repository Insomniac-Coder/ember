#$ test: compile-fail
#$ rules: R15, R3
## R3: `thread_shared` only on a value already `thread_shared` (E7017); a plain `log: Log` lends an
## object whose owner may be one-thread (AN-75).

import std.thread
from std.sync import Lock

class Log:
    lock: Lock = Lock()
    lines: Array[String] guarded by lock = []

    fn push(s: StrView) changes lines:
        with this.lock:
            this.lines.push(String(s))

fn start(log: Log) changes Log:
    a = thread.spawn(fn() with thread_shared log changes Log: log.push("a"))    #$ error[E7017]: `log` is lent to `start`, and its owner may not be shared between threads; take it as `fn start(log: thread_shared Log) changes Log:`, and create the log shared in `main`: `start(thread_shared Log())`
    a.join()

fn main() changes Log:
    start(Log())
