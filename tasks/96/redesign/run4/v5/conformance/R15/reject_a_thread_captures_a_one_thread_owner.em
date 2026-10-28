#$ test: compile-fail
#$ rules: THR-1, THR-2, THR-10, CLO-16, FFI-22
## design rules: R15, R9, R12
## R15 (verdict 94): "`thread.spawn` takes a function any thread may run ..., so a function holding a
## one-thread value is refused by R12's rule (E7015)"; `shares log` is such a value (AN-75). The nearest
## allowed program is `accept_threads_share_a_log_and_push_under_its_lock.em`.

import std.thread
from std.sync import Lock

class Log:
    lock: Lock = Lock()
    lines: Array[String] guarded by lock = []

    fn push(s: StrView):
        with this.lock:
            this.lines.push(String(s))

fn start(log: thread_shared Log):
    a = thread.spawn(fn() shares log: log.push("a"))    #$ error[E7015]: `shares log` makes an owner of `log` that stays on one thread, and `thread.spawn` runs this function on a new one; write `a = thread.spawn(fn() thread_shared log: log.push("a"))` in place of line 19
    a.join()

fn main():
    start(thread_shared Log())
