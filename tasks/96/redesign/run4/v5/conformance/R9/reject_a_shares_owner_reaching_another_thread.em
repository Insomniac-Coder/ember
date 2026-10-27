#$ test: compile-fail
#$ rules: R9, R15
## R9: "a `shares` value never reaches another thread (E7015)": its count is a plain add, so two
## threads would change one count (AN-71's refused `share_cached`).

import std.thread

class Cache:
    hits: int = 0

class Names:
    items: Array[String] = []
    cache: shares Cache

fn share_cached():
    c = shares Cache()
    names = Names(cache=shares c)
    h = thread.spawn(fn() owns names: names.items.len())    #$ error[E7015]: the thread gets `names`, which holds `cache`, and `cache`'s other owner `c` (line 16) stays on this thread; `shares` owners count without atomics, so two threads would change one count; share the cache between threads: write `cache: thread_shared Cache` in `Names`, `c = thread_shared Cache()` instead of line 16 and `names = Names(cache=thread_shared c)` instead of line 17
    println(c.hits, h.join())

fn main():
    share_cached()
