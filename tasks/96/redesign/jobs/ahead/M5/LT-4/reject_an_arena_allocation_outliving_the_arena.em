# Clause LT-4.5: "Arena allocations point into the arena (`[ARN-1]`)."
#$ test: compile-fail
#$ rules: LT-4, ARN-1, ARN-3
# A view `alloc` gives points into the arena: returned past a local arena it is `E3063`, and after
# `reset()` its use is `E3060` (`[ARN-1]`). The spec gives no message text for either: codes only.

fn escaped() -> ref mut int:
    frame = Arena.with_capacity(64)
    return frame.alloc(owns 1)    #$ error[E3063]

fn reused():
    frame = Arena.with_capacity(64)
    r = frame.alloc(owns 5)
    frame.reset()
    println(r)    #$ error[E3060]

fn main():
    println(escaped())
    reused()
