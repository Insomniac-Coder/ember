#$ test: compile-fail
#$ rules: FFI-22
#$ error[E7015]: the callback `tick` captures `counter`
#$ profiles: debug
# `[FFI-22]` — a callback C may call from any thread holds only `Send` values: a one-thread
# owner is refused, unless the contract says `threads(main)` or `threads(creator)`.

# SAFETY: the test's own C, which runs nothing before `main`
unsafe import c "./ticks.c" with (overlay = "overlays/any.em") as ticks

class Counter:
    count: i32 = 0

fn main():
    counter = shares mut Counter()
    ticks.tick_here(fn(): counter.count += 1)
    println(counter.count)
