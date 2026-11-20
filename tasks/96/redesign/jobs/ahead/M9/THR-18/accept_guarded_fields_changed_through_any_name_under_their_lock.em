# Clause THR-18.3: "`init` sets guarded fields without the lock"; "it is changed under its lock
# through any name that reaches its object, a plain one included, with no word at the call"; "An
# object whose every field that may change is `guarded by` its lock (...) may reach one call through
# any number of names, a plain argument and a function value that holds it included"
#$ test: run-pass
#$ rules: THR-18, BRW-11, THR-1
#$ stdout: 1
#$ stdout: 2
#$ stdout: 2
#$ stdout: 3
# `init` sets `lines` with no lock; `add_line` changes it through a plain parameter; a function
# that needs the lock is called inside it; `dump` gets the log as a plain argument and inside a
# function value (`[BRW-11]`'s exception 2).

from std.sync import Lock

class Log:
    lock: Lock = Lock()
    lines: Array[String] guarded by lock

    fn init():
        this.lines = [String("start")]

    fn push(s: StrView):
        with this.lock:
            this.lines.push(owns String(s))

fn add_line(log: Log):
    with log.lock:
        log.lines.push(owns String("hi"))

fn count_raw(log: Log) -> int:
    return log.lines.len()

fn dump(l: thread_shared Log, owns note: thread_shared fn()):
    with l.lock:
        println(l.lines.len())
    note()

fn main():
    log = Log()
    with log.lock:
        println(log.lines.len())
    add_line(log)
    with log.lock:
        println(count_raw(log))
    shared = thread_shared Log()
    shared.push("a")
    dump(shared, owns fn() thread_shared shared: shared.push("seen"))
    with shared.lock:
        println(shared.lines.len())
