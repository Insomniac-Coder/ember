#$ test: run-pass
#$ rules: FFI-22, FFI-21
#$ profiles: debug, development, release
#$ stdout: 2
# `[FFI-22]` — the compiler cannot see which thread C calls a callback from, so a callback with no
# `threads(...)` word is `threads(any)`: it may be called from any thread, and what it changes is
# changed by its own operations, here an `Atomic`'s.

from std.sync import Atomic
import c "./ticks.c" with (overlay = "overlays/any.em") as ticks

fn main():
    count: Atomic[i32] = Atomic(0)
    ticks.tick_here(fn(): count.fetch_add(1))
    ticks.tick_elsewhere(fn(): count.fetch_add(1))
    println(count.load())
