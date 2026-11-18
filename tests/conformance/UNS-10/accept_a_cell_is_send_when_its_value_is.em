#$ test: run-pass
#$ rules: UNS-10
#$ stdout: 5
# `[UNS-10]` — an `UnsafeCell[T]` is `Send` when `T` is: an `UnsafeCell[int]` moves to the thread whose
# function captures it, which reads it through the raw pointer `get` gives.

import std.thread
from std.mem import UnsafeCell

fn peek(c: UnsafeCell[int]) -> int:
    # SAFETY(aliasing): the thread holds the cell alone, and nothing writes it
    unsafe:
        return read(c.get(), 0)

fn main():
    cell = UnsafeCell(5)
    t = thread.spawn(owns owned fn() -> int => peek(cell))
    println(t.join())
