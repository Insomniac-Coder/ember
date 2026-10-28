#$ test: run-pass
#$ rules: R12, R5, R15
#$ stdout: 1
## Verdict 44, R4 and R5 (rules.md lines 247-251): `fn add_line(log: Log) changes Log: log.push("hi")`,
## the verdict's own line: no word at the call, the change named by type. `main` names `Log` as AN-75's
## `main` does for a log it made itself.

from std.sync import Lock

class Log:
    lock: Lock = Lock()
    lines: Array[String] guarded by lock = []

    fn push(s: StrView) changes lines:
        with this.lock:
            this.lines.push(String(s))

fn add_line(log: Log) changes Log:
    log.push("hi")

fn main() changes Log:
    log = Log()
    add_line(log)
    with log.lock:
        println(log.lines.len())
