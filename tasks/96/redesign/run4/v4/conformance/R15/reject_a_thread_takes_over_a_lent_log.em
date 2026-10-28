#$ test: compile-fail
#$ rules: R15, R3
## R3 and R15: a name that does not own the value cannot hand it over (E3013); `start`'s plain `log`
## is lent (AN-75).

import std.thread
from std.sync import Lock

class Log:
    lock: Lock = Lock()
    lines: Array[String] guarded by lock = []

    fn push(s: StrView) changes lines:
        with this.lock:
            this.lines.push(String(s))

fn start(log: thread_shared Log) changes Log:
    a = thread.spawn(fn() with owns log changes Log: log.push("a"))    #$ error[E3013]: `log` is lent to `start` (a plain parameter), so the thread cannot take it over (`with owns log`, line 18); to give each thread one more owner of the log, write `fn() with thread_shared log changes Log: log.push("a")`
    a.join()

fn main() changes Log:
    start(thread_shared Log())
