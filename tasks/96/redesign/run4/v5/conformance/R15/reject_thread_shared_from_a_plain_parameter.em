#$ test: compile-fail
#$ rules: THR-1
## design rules: R15, R3
## R3: `thread_shared` only on a value already `thread_shared` (E7017); a plain `log: Log` lends an
## object whose owner may be one-thread (AN-75). The nearest allowed program is
## `accept_threads_share_a_log_and_push_under_its_lock.em`.

import std.thread
from std.sync import Lock

class Log:
    lock: Lock = Lock()
    lines: Array[String] guarded by lock = []

    fn push(s: StrView):
        with this.lock:
            this.lines.push(String(s))

fn start(log: Log):
    a = thread.spawn(fn() thread_shared log: log.push("a"))    #$ error[E7017]: `log` is lent to `start`, and its owner may not be shared between threads; take it as `fn start(log: thread_shared Log):` in place of line 18, and create the log shared in `main`: `start(thread_shared Log())` in place of line 23
    a.join()

fn main():
    start(Log())
