#$ test: compile-fail
#$ rules: R15, R12
## R15 (verdicts 43 and 86): "A call through a function value ... that may run a function taking a lock
## of the kind this thread holds is refused, with no loop needed (E7018)": what `note` may run is worked
## out from the program (here the function `main` passes, whose `log.push` takes a `Log`'s lock; section
## 3's message in this program's lines). The two ways in to the log, `log` and the function holding it,
## are allowed by verdict 89 (R4, exception 2). The nearest allowed program is
## `accept_the_function_value_called_after_the_lock_is_let_go.em`.

from std.sync import Lock

class Log:
    lock: Lock = Lock()
    lines: Array[String] guarded by lock = []

    fn push(s: StrView):
        with this.lock:
            this.lines.push(String(s))

fn flush(log: thread_shared Log, note: fn()):
    with log.lock:
        println(log.lines.len())
        note()    #$ error[E7018]: `note()` may run the function `main` passes on line 27, which takes a `Log`'s lock (`log.push`, under `lock`, line 18), and this thread holds `log`'s lock, so taking it again would wait for ever; call `note()` after the `with log.lock:` block: move line 23 below the block

fn main():
    log = thread_shared Log()
    flush(log, fn() thread_shared log: log.push("seen"))
