#$ test: run-pass
#$ rules: STD-25
#$ stdout: 1546998764402558742 1 0.6800434110281394
#$ stdout: [2, 3, 5, 1, 4]
#$ stdout: 5
#$ stdout: true
# `[STD-25]` — `Rng.seeded(seed)` is xoshiro256** seeded by SplitMix64, integer arithmetic only, so
# seed 42 gives these numbers on every machine (the first three checked against the published
# algorithms): `int_in(a..b)` draws again rather than favour low values, `float()` is 53 bits
# scaled exactly into `[0, 1)`, `shuffle` is Fisher and Yates over a mutable view, `choice` refers
# to one of the items. `Rng.from_entropy()` seeds from the system's generator.

import std.random

fn main():
    rng = random.Rng.seeded(42)
    println(rng.next_u64(), rng.int_in(1..7), rng.float())
    xs = [1, 2, 3, 4, 5]
    rng.shuffle(xs.as_mut_span())
    println(xs)
    match rng.choice(xs.as_span()):
        Some(x):
            println(x)
        None:
            println("none")
    fresh = random.Rng.from_entropy()
    n = fresh.int_in(0..10)
    println(n >= 0 and n < 10)
