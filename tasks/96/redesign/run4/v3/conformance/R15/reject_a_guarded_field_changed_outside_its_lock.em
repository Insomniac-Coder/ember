#$ test: compile-fail
#$ rules: R15
## R15: a `guarded by` field is changed only while its lock is held (section 3's E7013 message; AN-74).

from std.sync import Lock

class Stats:
    lock: Lock = Lock()
    words: int guarded by lock = 0

fn count() -> int:
    stats = Stats()
    stats.words += 1    #$ error[E7013]: `words` is guarded by `lock`; change it inside `with stats.lock:`
    with stats.lock:
        return stats.words

fn main():
    println(count())
