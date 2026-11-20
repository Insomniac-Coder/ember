# Clause THR-1.2: "its state changes only through an `Atomic`, a `Mutex`, an `RwLock`, a `Sender`
# (...), or a field `guarded by` a lock, under that lock"; and the E7014 message's fix
#$ test: run-pass
#$ rules: THR-1, THR-10, THR-17
#$ stdout: 3
#$ stdout: 1
# Two threads each hold one more owner and add through an `Atomic`; a list filled where the object
# is made.

import std.thread
from std.sync import Atomic

class Stats:
    hits: Atomic[int] = Atomic(0)

class Names:
    items: Array[String] = []

fn main():
    st = thread_shared Stats()
    a = thread.spawn(owns fn() thread_shared st: st.hits.fetch_add(1))
    b = thread.spawn(owns fn() thread_shared st: st.hits.fetch_add(2))
    a.join()
    b.join()
    println(st.hits.load())
    names = thread_shared Names(items=[String("a")])
    println(names.items.len())
