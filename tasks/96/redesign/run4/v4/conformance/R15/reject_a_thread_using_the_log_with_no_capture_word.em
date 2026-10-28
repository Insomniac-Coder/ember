#$ test: compile-fail
#$ rules: R15
## R15: an unscoped thread names what it holds (E7004); `with owns log` is not offered, since `start`
## does not own the log (AN-75).

import std.thread
from std.sync import Lock

class Log:
    lock: Lock = Lock()
    lines: Array[String] guarded by lock = []

    fn push(s: StrView) changes lines:
        with this.lock:
            this.lines.push(String(s))

fn start(log: thread_shared Log) changes Log:
    a = thread.spawn(fn() changes Log: log.push("a"))    #$ error[E7004]: this thread may outlive `start`, so it cannot use `start`'s `log`; say what the thread holds: `fn() with thread_shared log changes Log: log.push("a")` (one more owner of the log)
    a.join()

fn main() changes Log:
    start(thread_shared Log())
