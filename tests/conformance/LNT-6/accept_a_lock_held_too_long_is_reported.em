#$ test: run-pass
#$ command: lint
#$ rules: LNT-6, THR-3
#$ stdout: 6
#$ stdout: 1
## `[LNT-6]` — `L3002`: a guard still held across a call after its last use; `L7001`: a lock held
## across a call that may block. A `with` block that ends at the guard's last use holds it no
## longer than it must.

import std.sync
import std.time

fn work(n: int) -> int:
    return n * 2

fn main():
    m = sync.Mutex(0)
    with g = m.lock():
        g += 1
        println(work(3))  #$ warning[L3002]: `g` is still held here, after its last use
        time.sleep(time.Duration.from_millis(1))  #$ warning[L7001]: the lock `g` guards is held across a call that may block
    with g = m.lock():
        println(g)
