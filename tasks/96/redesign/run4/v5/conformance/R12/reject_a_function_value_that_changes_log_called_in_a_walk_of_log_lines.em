#$ test: compile-fail
#$ rules: R12, R15, R18
#$ help: move line 27 below the block
## Verdicts 43, 73 and 86, R15 (rules.md lines 881-890), R8 (c) (lines 492-494), R18 (lines 954-957): a
## call through a function value that may run a function taking a `Log`'s lock (worked out from the
## function stored in `note`), inside a walk of a log's list under its lock, would take the lock this
## thread holds; the lock error is shown, not the walk's (E3020), whose fix (after the loop, still inside
## the lock) would still wait for ever. rules.md section 3's line 1099 third message, in this program's
## lines. `note` is made inside `dump`; passing it beside `log` would be allowed too (verdict 89, R4
## exception 2), but this test keeps one fault.

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
            note()    #$ error[E7018]: this loop walks `log.lines` (line 25) under `log`'s lock, and `note()` (line 27) may run the function stored on line 23, which takes a `Log`'s lock (`log.push`, under `lock`), so it would wait for the lock this thread holds; call `note()` after the `with log.lock:` block: move line 27 below the block

fn main():
    log = thread_shared Log()
    log.push("a")
    dump(log)
