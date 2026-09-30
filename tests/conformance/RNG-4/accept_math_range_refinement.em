#$ test: run-pass
#$ rules: RNG-4, RNG-10, RNG-5a
#$ profiles: debug, release, shipping
#$ stdout: 42
# `[RNG-4]` — the prelude's `min`, `max` and `clamp` carry what is known of
# their operands' ranges (D-142; they replaced `std.math`'s per-type copies):
# a clamp to the bounds, or a `max`/`min` chain over a proven range, gives a
# value whose range fits, which `[RNG-10]`(d) converts without a check. A
# float's fact comes only from a comparison (`[RNG-4a]`, D-392), so a float
# clamp gives none: `Ratio.clamped` is the float form.

type Percent = i32 in 0 ..= 100

fn clamp_percent(value: i32) -> Percent:
    return clamp(value, 0, 100)

fn min_max_percent(value: i32) -> Percent:
    if value >= 0:
        lower = max(value, 0)
        bounded = min(lower, 100)
        return bounded
    return Percent.clamped(0)

fn main():
    first: i32 = clamp_percent(42)
    second: i32 = min_max_percent(42)
    println(first + second - 42)
