#$ test: run-pass
#$ rules: DET-10
#$ stdout: changed
#$ stdout: 0
# `[DET-10]` — the runtime establishes the defined floating-point environment on every thread it
# creates. Foreign code sets this thread's rounding upward (outside the guarantee); the thread
# `thread.spawn` starts rounds to nearest (`fegetround()` is `FE_TONEAREST`, 0) all the same, though
# a POSIX thread starts with its creator's environment. Upward is `0x800` on x86 glibc and `0x100`
# with the Microsoft C runtime; each refuses the other's value.

import std.thread

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    safe fn fesetround(mode: i32) -> i32
    safe fn fegetround() -> i32

fn main():
    fesetround(0x800)
    fesetround(0x100)
    if fegetround() != 0:
        println("changed")
    worker = thread.spawn(owned fn() => fegetround())
    println(worker.join())
