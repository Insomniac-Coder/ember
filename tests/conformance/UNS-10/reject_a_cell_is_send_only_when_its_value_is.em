#$ test: compile-fail
#$ rules: UNS-10, THR-8
# `[UNS-10]` — an `UnsafeCell[T]` is `Send` only when `T` is: one holding a `Shared[int]` (one thread's
# count) does not move to another thread.

import std.thread
from std.mem import UnsafeCell

fn size(c: UnsafeCell[shares int]) -> int:
    return 1

fn main():
    held = UnsafeCell(shares 1)
    t = thread.spawn(owns owned fn() -> int => size(held))    #$ error[E7015]: the thread's function captures `held`, a `std.mem.UnsafeCell[shares i64]`, which is not Send
    println(t.join())
