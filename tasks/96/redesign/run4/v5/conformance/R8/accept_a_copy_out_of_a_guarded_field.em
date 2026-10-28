#$ test: run-pass
#$ rules: BRW-19
#$ stdout: hi
## design rules: R8, R15
## R8 (e): what leaves `with log.lock:` is a `String` of your own, not a view of the guarded list. The
## nearest program to `reject_a_view_of_a_guarded_field_returned_out_of_its_lock.em` that is allowed.

from std.sync import Lock

class Log:
    lock: Lock = Lock()
    lines: Array[String] guarded by lock = []

fn first_line(log: Log) -> String:
    with log.lock:
        return String(log.lines[0])

fn main():
    log = Log()
    with log.lock:
        log.lines.push(String("hi"))
    println(first_line(log))
