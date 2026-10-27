#$ test: compile-fail
#$ rules: R15, R12
## R15: "A call through a function value whose type says it changes a type whose lock this thread holds
## (`note()`, a `fn() changes Log`, inside `with log.lock:`) is refused, with no loop needed (E7018)"
## (verdict 43; section 3's message).

from std.sync import Lock

class Log:
    lock: Lock = Lock()
    lines: Array[String] guarded by lock = []

    fn push(s: StrView) changes lines:
        with this.lock:
            this.lines.push(String(s))

fn flush(log: thread_shared Log, note: fn() changes Log) changes Log:
    with log.lock:
        println(log.lines.len())
        note()    #$ error[E7018]: `note()` may change any `Log`, this one included (its type says `changes Log`), and this thread holds `log`'s lock, so taking it again inside `note` would wait for ever; call `note()` after the `with log.lock:` block

fn main() changes Log:
    log = thread_shared Log()
    flush(log, fn() with thread_shared log changes Log: log.push("seen"))
