#$ test: compile-fail
#$ rules: STD-23, DET-2
#$ error[E4070]: `@deterministic` function `stamp` reaches a nondeterministic operation
#$ error[E4003]: `@noblock` function `pause` reaches an operation that may wait
# `[STD-23]` — reading the monotonic clock is `Nondet`, and `time.sleep` carries `Block`.

import std.time
from std.time import Duration, Instant

@deterministic
fn stamp() -> int:
    return Instant.now().elapsed().as_nanos()

@noblock
fn pause():
    time.sleep(Duration.from_millis(1))

fn main():
    println(stamp())
    pause()
