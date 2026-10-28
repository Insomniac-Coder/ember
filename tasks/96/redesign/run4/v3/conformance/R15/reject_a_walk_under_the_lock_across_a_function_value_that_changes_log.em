#$ test: compile-fail
#$ rules: R15, R8, R12
## R8 (c) and R12 (verdict 29): a call through a `fn() changes Log` inside a walk of `log.lines` under
## its lock is refused when compiling (section 3's E3020 row; AN-75). E7018 also names this call; which
## of the two it gets is open (see the matrix, R15-9).

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
            note()    #$ error[E3020]: this loop walks `log.lines` (line 19) under `log`'s lock, and `note()` (line 21) may change any `Log`, this one included (its type says `changes Log`), so it would change the list during the walk and wait for the lock this thread holds; call `note()` after the `with log.lock:` block

fn main() changes Log:
    log = thread_shared Log()
    log.push("a")
    dump(log, fn() with thread_shared log changes Log: log.push("seen"))
