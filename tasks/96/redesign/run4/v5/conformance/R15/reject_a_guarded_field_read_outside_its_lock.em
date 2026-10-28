#$ test: compile-fail
#$ rules: THR-18
## design rules: R15
## R15: the lock "must be held for every read and change outside `init` (E7013)". The rules give the
## message for a change; this one, for a read, is written to R18.

from std.sync import Lock

class Stats:
    lock: Lock = Lock()
    words: int guarded by lock = 0

fn count() -> int:
    stats = Stats()
    with stats.lock:
        stats.words += 1
    return stats.words    #$ error[E7013]: `words` is guarded by `lock`; read it inside `with stats.lock:`

fn main():
    println(count())
