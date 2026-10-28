#$ test: compile-fail
#$ rules: R12, R8, R15
#$ help: call `note()` after the `with log.lock:` block
## Verdict 29's gain, R8 (c) and R12 (rules.md lines 357-361, 553-556): a call through a function value
## whose type says `changes Log`, inside a walk of a log's list under its lock, may change the list and
## wait for the lock this thread holds. AN-75 part 6's program; rules.md line 837's E3020 message.

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
            note()    #$ error[E3020]: this loop walks `log.lines` (line 20) under `log`'s lock, and `note()` (line 22) may change any `Log`, this one included (its type says `changes Log`), so it would change the list during the walk and wait for the lock this thread holds; call `note()` after the `with log.lock:` block

fn main() changes Log:
    log = thread_shared Log()
    log.push("a")
    dump(log, fn() with thread_shared log changes Log: log.push("seen"))
