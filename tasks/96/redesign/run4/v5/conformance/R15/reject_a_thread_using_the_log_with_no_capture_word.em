#$ test: compile-fail
#$ rules: R15
## R15: an unscoped thread names what it holds (E7004); `owns log` is not offered, since `start` does not
## own the log (AN-75). The nearest allowed program is
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
    a = thread.spawn(fn(): log.push("a"))    #$ error[E7004]: this thread may outlive `start`, so it cannot use `start`'s `log`; say what the thread holds: write `a = thread.spawn(fn() thread_shared log: log.push("a"))` in place of line 19 (one more owner of the log)
    a.join()

fn main():
    start(thread_shared Log())
