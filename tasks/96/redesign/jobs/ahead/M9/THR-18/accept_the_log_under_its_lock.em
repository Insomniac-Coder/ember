# Clause THR-18.1: "A `Lock` is a lock with no value inside; `with x.lock:` holds it for the block";
# "**A direct call that takes a lock this thread already holds enters the callee past it**, without
# taking it again, only where the compiler proves it is the very lock held"
#$ test: run-pass
#$ rules: THR-18, THR-19
#$ stdout: 1
# The spec's own program: `log.push` takes `this.lock`, which is `log`'s, held already.

from std.sync import Lock

class Log:
    lock: Lock = Lock()
    lines: Array[String] guarded by lock = []

    fn push(s: StrView):
        with this.lock:
            this.lines.push(owns String(s))

fn main():
    log = Log()
    with log.lock:
        log.push("a")
        println(log.lines.len())
