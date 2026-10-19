#$ test: run-pass
#$ rules: STD-23
#$ stdout: 1500000000 1500 true
#$ stdout: true
# `[STD-23]` — a `Duration` is a whole count of nanoseconds, with `+` and `-`; `time.sleep(d)`
# waits at least `d`, and the monotonic clock never goes back, so the time it measured is no less.

import std.time
from std.time import Duration, Instant

fn main():
    d = Duration.from_secs(1) + Duration.from_millis(500)
    println(d.as_nanos(), d.as_millis(), d.as_secs_f64() == 1.5)
    start = Instant.now()
    time.sleep(Duration.from_millis(20))
    println(start.elapsed().as_nanos() >= 20000000)
