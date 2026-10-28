#$ test: compile-fail
#$ rules: R15, R9
## R15: "`with shares`, `with shares mut` and `with weakref` only where the function stays on this
## thread"; a one-thread owner reaching another thread is E7015 (AN-75).

import std.thread
from std.sync import Lock

class Log:
    lock: Lock = Lock()
    lines: Array[String] guarded by lock = []

    fn push(s: StrView) changes lines:
        with this.lock:
            this.lines.push(String(s))

fn start(log: thread_shared Log) changes Log:
    a = thread.spawn(fn() with shares log changes Log: log.push("a"))    #$ error[E7015]: `with shares log` makes an owner of `log` that stays on one thread, and this function runs on a new one; write `fn() with thread_shared log changes Log: log.push("a")`
    a.join()

fn main() changes Log:
    start(thread_shared Log())
