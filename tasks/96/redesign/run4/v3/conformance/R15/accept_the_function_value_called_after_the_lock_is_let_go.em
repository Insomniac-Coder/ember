#$ test: run-pass
#$ rules: R15, R12
#$ stdout: 0
#$ stdout: 1
## R15: the line E7018 gives: `note()` after the `with log.lock:` block (verdict 43).

from std.sync import Lock

class Log:
    lock: Lock = Lock()
    lines: Array[String] guarded by lock = []

    fn push(s: StrView) changes lines:
        with this.lock:
            this.lines.push(String(s))

fn flush(log: thread_shared Log, note: fn() changes Log) changes Log:
    with log.lock:
        println(log.lines.len())
    note()
    with log.lock:
        println(log.lines.len())

fn main() changes Log:
    log = thread_shared Log()
    flush(log, fn() with thread_shared log changes Log: log.push("seen"))
