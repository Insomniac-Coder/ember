#$ test: run-pass
#$ rules: R15, R4, R12
#$ stdout: a
#$ stdout: 2
## R4, exception 2 (verdict 89): "A `thread_shared` object whose every field that may change is `guarded
## by` its lock ... may reach one call through any number of names, a plain argument and a function
## value that holds it included: `dump(log, fn() thread_shared log: log.push("seen"))`, whose `dump`
## reads the lines inside `with log.lock:` and calls `note()` after the block, is allowed": the lock
## keeps the two ways in apart, as Rust allows with `&Log` and a `Mutex`. Also the line E7018 gives in
## `reject_a_walk_under_the_lock_across_a_function_value_that_changes_log.em`.

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
    with log.lock:
        println(log.lines.len())

fn main():
    log = thread_shared Log()
    log.push("a")
    dump(log, fn() thread_shared log: log.push("seen"))
