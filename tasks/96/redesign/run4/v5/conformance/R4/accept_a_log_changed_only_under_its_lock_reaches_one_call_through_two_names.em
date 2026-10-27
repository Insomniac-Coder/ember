#$ test: run-pass
#$ rules: R4, R15, R12
#$ stdout: a
#$ stdout: 2
## R4 (overlap, exception 2), R15: a `thread_shared` object whose changes go only through its lock may
## reach one call through any number of names (verdict 89): `log` is a plain argument of `dump` and is
## held by the function value given beside it, and the lock keeps the two ways in apart, as Rust allows
## the same with `&Log` and a `Mutex`. `dump` calls `note()` after the `with log.lock:` block; inside it
## the call would be refused (E7018, `R15/reject_a_call_through_a_function_value_that_may_take_the_held_lock`).

from std.sync import Lock

class Log:
    lock: Lock = Lock()
    lines: Array[String] guarded by lock = []

    fn push(s: StrView):
        with this.lock:
            this.lines.push(String(s))

fn dump(log: thread_shared Log, note: fn()):
    with log.lock:
        for line in log.lines:
            println(line)
    note()

fn main():
    log = thread_shared Log()
    log.push("a")
    dump(log, fn() thread_shared log: log.push("seen"))
    with log.lock:
        println(log.lines.len())
