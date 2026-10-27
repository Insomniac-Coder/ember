#$ test: compile-fail
#$ rules: THR-19
## design rules: R15, R12
## R15 (verdict 43): "The compiler cannot tell which `Log` a stored function holds, so a function that
## takes another `Log`'s lock is refused too; the way out moves one line" (section 3's E7018 message, in
## this program's lines).

from std.sync import Lock

class Log:
    lock: Lock = Lock()
    lines: Array[String] guarded by lock = []

    fn push(s: StrView):
        with this.lock:
            this.lines.push(String(s))

fn flush(log: thread_shared Log, note: fn()):
    with log.lock:
        println(log.lines.len())
        note()    #$ error[E7018]: `note()` may run the function `main` passes on line 25, which takes a `Log`'s lock (`other.push`, under `lock`, line 15), and this thread holds `log`'s lock, so taking it again could wait for ever; call `note()` after the `with log.lock:` block: move line 20 below the block

fn main():
    log = thread_shared Log()
    other = thread_shared Log()
    flush(log, fn() thread_shared other: other.push("seen"))
