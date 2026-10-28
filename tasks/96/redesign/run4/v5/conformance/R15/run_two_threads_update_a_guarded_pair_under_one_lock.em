#$ test: run-pass
#$ rules: THR-1, RC-1, CLO-1
#$ stdout: 8 5
## design rules: R15, R9, R12
## R15: the read and the write of `longest` sit under one lock, so it ends as 5 whichever thread runs
## first; "a lock: where the program takes one" is all that runs (section 2; AN-80's NEW, with a line
## that prints the result).

import std.thread
from std.sync import Lock

class Stats:
    lock: Lock = Lock()
    words: int guarded by lock = 0
    longest: int guarded by lock = 0

    fn note(n: int):
        with this.lock:
            this.words += n
            if n > this.longest:
                this.longest = n

fn main():
    st = thread_shared Stats()
    a = thread.spawn(fn() thread_shared st: st.note(3))
    st.note(5)
    a.join()
    with st.lock:
        println(st.words, st.longest)
