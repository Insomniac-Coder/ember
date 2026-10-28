#$ test: run-pass
#$ rules: R15, R9, R12
#$ stdout: 3
## R15: no mark: the value is created `thread_shared` where it is shared, and its `Atomic` field is
## changed by its own operations from any name (R5); the thread's function names `changes Stats`
## (verdict 29) and `main` names it too (verdict 30).

import std.thread
from std.sync import Atomic

class Stats:
    words: Atomic[int] = Atomic(0)

fn main() changes Stats:
    st = thread_shared Stats()
    a = thread.spawn(fn() with thread_shared st changes Stats: st.words.fetch_add(2))
    st.words.fetch_add(1)
    a.join()
    println(st.words.load())
