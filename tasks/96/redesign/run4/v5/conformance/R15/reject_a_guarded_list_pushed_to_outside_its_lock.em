#$ test: compile-fail
#$ rules: R15
## R15: "`guarded by lock` ... names the `Lock` that must be held for every read and change outside
## `init` (E7013)" (AN-75's refused `start`).

from std.sync import Lock

class Log:
    lock: Lock = Lock()
    lines: Array[String] guarded by lock = []

    fn push(s: StrView):
        with this.lock:
            this.lines.push(String(s))

fn start(log: thread_shared Log):
    log.lines.push(String("c"))    #$ error[E7013]: `lines` is guarded by `lock`; change it inside `with log.lock:`, or call the method that takes the lock: write `log.push("c")` in place of line 17

fn main():
    start(thread_shared Log())
