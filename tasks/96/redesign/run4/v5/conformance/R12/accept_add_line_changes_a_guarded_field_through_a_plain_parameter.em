#$ test: run-pass
#$ rules: R12, R4, R5, R15
#$ stdout: 1
## Verdict 44 ("Keep guarded by and the same way as mutex"), R4 (rules.md lines 265-269: "`fn
## add_line(log: Log): log.push("hi")`"), R5 (lines 405-410): a `guarded by` field is changed through a
## plain name inside its lock, with no word at the call; what `add_line` changes is worked out from its
## body (verdict 86), so its first line says nothing.

from std.sync import Lock

class Log:
    lock: Lock = Lock()
    lines: Array[String] guarded by lock = []

    fn push(s: StrView):
        with this.lock:
            this.lines.push(String(s))

fn add_line(log: Log):
    log.push("hi")

fn main():
    log = Log()
    add_line(log)
    with log.lock:
        println(log.lines.len())
