#$ test: run-fail
#$ rules: FFI-33b, FFI-21
#$ profiles: debug
#$ panics: a callback whose contract says `threads(creator)` is called on thread
# `[FFI-33b]` — in the debug profile, a callback contracted `threads(creator)` that C calls on
# another thread panics, naming both threads, before it runs.

import c "./ticks.c" with (overlay = "overlays/ticks.em") as ticks

class Counter:
    count: i32 = 0

fn main():
    counter = Counter()
    ticks.tick_elsewhere(fn(): counter.count += 1)
    println(counter.count)
