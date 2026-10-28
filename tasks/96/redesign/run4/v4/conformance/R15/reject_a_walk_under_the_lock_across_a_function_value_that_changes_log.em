#$ test: compile-fail
#$ rules: R15, R8, R12
## R8 (c), R15 and R18 (verdict 73): "A call through a function value that may take a lock this thread
## holds, inside a walk under that lock, is E7018, not E3020": E3020's fix (after the loop, still inside
## the lock) would still wait for ever (section 3's E7018 "in a walk" message, in this program's lines;
## AN-75).

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
            note()    #$ error[E7018]: this loop walks `log.lines` (line 20) under `log`'s lock, and `note()` (line 22) may change any `Log`, this one included (its type says `changes Log`), so it would wait for the lock this thread holds; call `note()` after the `with log.lock:` block

fn main() changes Log:
    log = thread_shared Log()
    log.push("a")
    dump(log, fn() with thread_shared log changes Log: log.push("seen"))
