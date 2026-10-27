#$ test: compile-fail
#$ rules: R12, R5, R15
#$ help: fn() with thread_shared log changes Log: log.push("a")
## R12 (rules.md lines 605-608), verdict 29: a change made under a lock is named like every change, by
## the type of what changes (`changes Log`). AN-75 part 6, v2's line; the message is rules.md section 3's
## E2105 row (line 989), word for word.

import std.thread
from std.sync import Lock

class Log:
    lock: Lock = Lock()
    lines: Array[String] guarded by lock = []

    fn push(s: StrView) changes lines:
        with this.lock:
            this.lines.push(String(s))

fn start(log: thread_shared Log) changes Log:
    a = thread.spawn(fn() with thread_shared log: log.push("a"))    #$ error[E2105]: this function changes `log` through `push` (`changes lines`, under `log`'s lock); say so on its first line: `fn() with thread_shared log changes Log: log.push("a")`
    b = thread.spawn(fn() with thread_shared log changes Log: log.push("b"))
    a.join()
    b.join()

fn main() changes Log:
    start(thread_shared Log())
