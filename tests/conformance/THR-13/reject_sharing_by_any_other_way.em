#$ test: compile-fail
#$ rules: THR-13, THR-1, HEAP-10, THR-10
#$ error[E7003]: cannot write field `n` of an `@sync` class after `init` or after using `self` as a whole
#$ error[E1010]: `SyncShared[i64]` has no method named `get_mut`
#$ error[E7004]: the thread's function captures `count`, a `Shared[i64]`, which is not Send
# `[THR-13]` — no other way reaches a location from two threads: a `@sync` object's fields are not
# written after `init`, a `SyncShared` lends no `ref mut`, and a `Shared` does not cross.

import std.thread
from std.sync import SyncShared

@sync
class Fixed:
    n: int = 0

fn main():
    f = Fixed()
    f.n = 2
    s = SyncShared(5)
    s.get_mut()
    count = Shared(0)
    t = thread.spawn(owned fn() => count.get() + 1)
