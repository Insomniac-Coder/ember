#$ test: compile-fail
#$ rules: R12, R5, R15
#$ help: fn add_line(log: Log) changes Log:
## Verdict 44 (`guarded by` follows the `Mutex`'s rules), R5 (rules.md lines 212-216, 305-308): a
## `guarded by` field is changed through a plain name inside its lock, and the change is named on the
## first line by the object's type. Message in the shape of rules.md section 3's E2105 lock message.

from std.sync import Lock

class Log:
    lock: Lock = Lock()
    lines: Array[String] guarded by lock = []

    fn push(s: StrView) changes lines:
        with this.lock:
            this.lines.push(String(s))

fn add_line(log: Log):    #$ error[E2105]: `add_line` changes `log` through `push` (`changes lines`, under `log`'s lock); add it to its first line: `fn add_line(log: Log) changes Log:`
    log.push("hi")

fn main() changes Log:
    log = Log()
    add_line(log)
    with log.lock:
        println(log.lines.len())
