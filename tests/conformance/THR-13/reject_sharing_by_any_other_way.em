#$ test: compile-fail
#$ rules: THR-13, THR-1, HEAP-10, THR-10
# `[THR-13]` — no other way reaches a place from two threads: no owner writes a `thread_shared` value,
# it reads as its value with no way to change it, and a `shares` owner does not cross.

import std.thread

class Fixed:
    n: int = 0

fn main():
    f = thread_shared Fixed()
    f.n = 2    #$ error[E7014]: is shared between threads
    s = thread_shared 5
    s.get_mut()    #$ error[E2072]: a `thread_shared i64` reads as an `i64`; remove `.get_mut()`
    count = shares mut 0
    _t = thread.spawn(owns owned fn() => count + 1)    #$ error[E7015]: captures `count`, a `shares mut i64`
