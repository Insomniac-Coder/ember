#$ test: run-pass
#$ rules: R12, R8, R15
#$ stdout: a
#$ stdout: 2
## R15 (rules.md lines 881-890), R12 (lines 717-720): the walk of `log.lines` ends, the lock is let go,
## and then `note()` runs, as the E7018 message of the nearest refused program says
## (reject_a_function_value_that_changes_log_called_in_a_walk_of_log_lines).

from std.sync import Lock

class Log:
    lock: Lock = Lock()
    lines: Array[String] guarded by lock = []

    fn push(s: StrView):
        with this.lock:
            this.lines.push(String(s))

fn dump(log: thread_shared Log):
    note = fn() thread_shared log: log.push("seen")
    with log.lock:
        for line in log.lines:
            println(line)
    note()

fn main():
    log = thread_shared Log()
    log.push("a")
    dump(log)
    with log.lock:
        println(log.lines.len())
