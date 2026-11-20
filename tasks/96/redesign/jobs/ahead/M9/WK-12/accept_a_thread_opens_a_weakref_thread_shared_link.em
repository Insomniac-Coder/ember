# Clause WK-12.1 with `[RC-5]` point 2: "A name opened from a `weakref thread_shared T` link
# (`[WK-11]`) holds one count for its life, as Rust's `Weak::upgrade` does (`[WK-12]`)"
#$ test: run-pass
#$ rules: WK-12, WK-11, RC-5, THR-2
#$ stdout: 1
#$ stdout: gone
# A thread opens a link to a `thread_shared` value and adds through its `Atomic`; after the last
# owner ends, the link finds nothing.

import std.thread
from std.sync import Atomic

class Stats:
    hits: Atomic[int] = Atomic(0)

fn hit(w: weakref thread_shared Stats):
    if w exists as s:
        s.hits.fetch_add(1)

fn main():
    st = thread_shared Stats()
    w = weakref st
    h = thread.spawn(owns fn() weakref st: hit(st))
    h.join()
    println(st.hits.load())
    st = thread_shared Stats()
    if w exists as s:
        println(s.hits.load())
    else:
        println("gone")
