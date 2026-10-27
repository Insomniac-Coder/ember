#$ test: run-pass
#$ rules: THR-1
#$ stdout: 5
## design rules: R15
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
