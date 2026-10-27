#$ test: compile-fail
#$ rules: R12, R15, R18
#$ help: call `note()` after the `with log.lock:` block
## Verdicts 29 and 73, R15 (rules.md lines 850-855), R18 (lines 906-909): a call through a function value
## whose type says `changes Log`, inside a walk of a log's list under its lock, would take the lock this
## thread holds; the lock error is shown, not the walk's (E3020), whose fix (after the loop, still inside the
## lock) would still wait for ever. rules.md section 3's line 1049 third message, in this program's lines.
## `note` is made inside `dump`, so no call is given a `Log` beside a `fn() changes Log` (an open question).

from std.sync import Lock

class Log:
    lock: Lock = Lock()
    lines: Array[String] guarded by lock = []

    fn push(s: StrView) changes lines:
        with this.lock:
            this.lines.push(String(s))

fn dump(log: thread_shared Log) changes Log:
    note = fn() with thread_shared log changes Log: log.push("seen")
    with log.lock:
        for line in log.lines:
            println(line)
            note()    #$ error[E7018]: this loop walks `log.lines` (line 23) under `log`'s lock, and `note()` (line 25) may change any `Log`, this one included (its type says `changes Log`), so it would wait for the lock this thread holds; call `note()` after the `with log.lock:` block

fn main() changes Log:
    log = thread_shared Log()
    log.push("a")
    dump(log)
