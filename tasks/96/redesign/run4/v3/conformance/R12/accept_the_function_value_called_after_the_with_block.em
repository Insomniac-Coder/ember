#$ test: run-pass
#$ rules: R12, R8, R15
#$ stdout: a
#$ stdout: 2
## R8 (c), R12 (rules.md lines 553-556): the walk of `log.lines` ends, the lock is let go, and then
## `note()` runs, as the E3020 message of the nearest refused program says.

from std.sync import Lock

class Log:
    lock: Lock = Lock()
    lines: Array[String] guarded by lock = []

    fn push(s: StrView) changes lines:
        with this.lock:
            this.lines.push(String(s))

fn dump(log: thread_shared Log, note: fn() changes Log) changes Log:
    with log.lock:
        for line in log.lines:
            println(line)
    note()

fn main() changes Log:
    log = thread_shared Log()
    log.push("a")
    dump(log, fn() with thread_shared log changes Log: log.push("seen"))
    with log.lock:
        println(log.lines.len())
