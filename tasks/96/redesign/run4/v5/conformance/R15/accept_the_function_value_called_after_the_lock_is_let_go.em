#$ test: run-pass
#$ rules: THR-1, CLO-1
#$ stdout: 0
#$ stdout: 1
## design rules: R15, R12
## R15: the line E7018 gives: `note()` after the `with log.lock:` block (verdict 43); `flush` is given the
## log and a function holding it, two ways in to a thing changed only through its lock, which R4 allows
## (verdict 89, exception 2).

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
    note()
    with log.lock:
        println(log.lines.len())

fn main():
    log = thread_shared Log()
    flush(log, fn() thread_shared log: log.push("seen"))
