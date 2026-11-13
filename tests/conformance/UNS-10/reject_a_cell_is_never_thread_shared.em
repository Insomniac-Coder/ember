#$ test: compile-fail
#$ rules: UNS-10, THR-9
# `[UNS-10]` — an `UnsafeCell` is never `ThreadShared`, even of an `int`: two tasks may not read a value
# holding one at once, since nothing checks what its raw pointer is used for.

import std.thread
from std.mem import UnsafeCell

struct Hits:
    count: UnsafeCell[int]

    fn read() -> int:
        # SAFETY(aliasing): nothing writes the count while it is read
        unsafe:
            return read(this.count.get(), 0)

fn main():
    hits = Hits(count=UnsafeCell(1))
    with scope = thread.scope():
        a = scope.spawn(owns fn() => hits.read())    #$ error[E7005]: the task borrows `hits`, a `Hits`
        b = scope.spawn(owns fn() => hits.read())    #$ error[E7005]: the task borrows `hits`, a `Hits`
