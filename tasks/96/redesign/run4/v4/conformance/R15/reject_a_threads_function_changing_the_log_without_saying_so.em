#$ test: compile-fail
#$ rules: R15, R12
## R12 (verdict 29): "a change made under a lock ... named by the type of what changes" (AN-75's
## refused v2 line).

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
    a.join()

fn main() changes Log:
    start(thread_shared Log())
