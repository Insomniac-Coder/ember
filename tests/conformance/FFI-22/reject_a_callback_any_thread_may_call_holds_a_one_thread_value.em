#$ test: compile-fail
#$ rules: FFI-22
#$ error[E7004]: the callback `tick` captures `counter`, a `ref mut Counter`, which is not Send
#$ profiles: debug
# `[FFI-22]` — a callback C may call from any thread holds only `Send` values: a class handle
# (a one-thread owner) is refused, unless the contract says `threads(main)` or `threads(creator)`.

import c "./ticks.c" with (overlay = "overlays/any.em") as ticks

class Counter:
    count: i32 = 0

fn main():
    counter = Counter()
    ticks.tick_here(fn(): counter.count += 1)
    println(counter.count)
