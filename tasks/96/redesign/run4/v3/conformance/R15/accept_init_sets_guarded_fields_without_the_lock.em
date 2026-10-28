#$ test: run-pass
#$ rules: R15
#$ stdout: 5
## R15: the lock must be held "for every read and change outside `init`": `init` sets a guarded field
## with no lock.

from std.sync import Lock

class Stats:
    lock: Lock = Lock()
    words: int guarded by lock = 0

    fn init(start: int):
        this.words = start

fn main():
    stats = Stats(5)
    with stats.lock:
        println(stats.words)
