#$ test: run-pass
#$ rules: FFI-33b, FFI-21, FFI-22
#$ profiles: debug, development, release
#$ stdout: 2
# `[FFI-33b]`, `[FFI-22]` — a callback whose contract says `threads(creator)`: what it captures
# need not be `Send` (a class handle here), and C calls it on the thread that made the call. The
# debug profile checks each call (the run-fail case beside this one calls it elsewhere).

import c "./ticks.c" with (overlay = "overlays/ticks.em") as ticks

class Counter:
    count: i32 = 0

fn main():
    counter = Counter()
    ticks.tick_here(fn(): counter.count += 1)
    ticks.tick_here(fn(): counter.count += 1)
    println(counter.count)
