#$ test: compile-fail
#$ rules: R8, R15
#$ help: return String(log.lines[0])
## R8 (e): the same for a `guarded by` field inside `with x.lock:`: a view of `log.lines` does not leave
## the block (row 4's message).

from std.sync import Lock

class Log:
    lock: Lock = Lock()
    lines: Array[String] guarded by lock = []

fn first_line(log: Log) -> StrView:
    with log.lock:
        return log.lines[0]    #$ error[E3063]: this points into the locked list, which can change once the lock ends; return a copy, or do the work inside the `with` block

fn main():
    log = Log()
    with log.lock:
        log.lines.push(String("hi"))
    println(first_line(log))
