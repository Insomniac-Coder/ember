#$ test: run-pass
#$ rules: R15
#$ stdout: 1
## R15: "A call whose body the compiler sees enters the callee past the lock it already holds (static
## re-entry)" (AN-74).

from std.sync import Lock

class Log:
    lock: Lock = Lock()
    lines: Array[String] guarded by lock = []

    fn push(s: StrView) changes lines:
        with this.lock:
            this.lines.push(String(s))

fn main():
    log = Log()
    with log.lock:
        log.push("a")
        println(log.lines.len())
