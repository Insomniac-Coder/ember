#$ test: compile-fail
#$ rules: R15, R3
## R3 and R15: a name that does not own the value cannot hand it over (E3013); `start`'s plain `log`
## is lent (AN-75). The nearest allowed program is
## `accept_threads_share_a_log_and_push_under_its_lock.em`.

import std.thread
from std.sync import Lock

class Log:
    lock: Lock = Lock()
    lines: Array[String] guarded by lock = []

    fn push(s: StrView):
        with this.lock:
            this.lines.push(String(s))

fn start(log: thread_shared Log):
    a = thread.spawn(fn() owns log: log.push("a"))    #$ error[E3013]: `log` is lent to `start` (a plain parameter), so the thread cannot take it over (`owns log`, line 19); to give the thread one more owner of the log, write `a = thread.spawn(fn() thread_shared log: log.push("a"))` in place of line 19
    a.join()

fn main():
    start(thread_shared Log())
