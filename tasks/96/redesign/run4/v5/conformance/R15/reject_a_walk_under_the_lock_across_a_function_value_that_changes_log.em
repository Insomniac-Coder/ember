#$ test: compile-fail
#$ rules: THR-19
## design rules: R15, R8, R12
## R8 (c), R15 and R18 (verdict 73): "A call through a function value that may take a lock this thread
## holds, inside a walk under that lock, is E7018, not E3020": E3020's fix (after the loop, still inside
## the lock) would still wait for ever (section 3's E7018 "in a walk" message, in this program's lines;
## AN-75). The two ways in to the log are allowed by verdict 89. The nearest allowed program is
## `accept_one_call_reaches_a_lock_guarded_log_through_two_names.em`.

from std.sync import Lock

class Log:
    lock: Lock = Lock()
    lines: Array[String] guarded by lock = []

    fn push(s: StrView):
        with this.lock:
            this.lines.push(String(s))

fn dump(log: thread_shared Log, note: fn()):
    with log.lock:
        for line in log.lines:
            println(line)
            note()    #$ error[E7018]: this loop walks `log.lines` (line 21) under `log`'s lock, and `note()` (line 23) may run the function `main` passes on line 28, which takes a `Log`'s lock (`log.push`, under `lock`, line 17), so it would wait for the lock this thread holds; call `note()` after the `with log.lock:` block: move line 23 below the block

fn main():
    log = thread_shared Log()
    log.push("a")
    dump(log, fn() thread_shared log: log.push("seen"))
