#$ test: run-pass
#$ rules: RNG-4, RNG-10, RNG-5a
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ 0.5
# `[RNG-4]` — the prelude's `min`, `max` and `clamp` carry what is known of
# their operands' ranges (D-142; they replaced `std.math`'s per-type copies):
# a clamp to the bounds, or a `max`/`min` chain over a proven range, gives a
# value whose range fits, which `[RNG-10]`(d) converts without a check. Over
# a range value itself they keep its type (`[RNG-5a]`), so the float chain
# starts from its representation.

type Percent = i32 in 0 ..= 100
type Ratio = f32 in 0.0 ..= 1.0
type NonNegative = f32 in 0.0 ..= 3.0

fn clamp_percent(value: i32) -> Percent:
    return clamp(value, 0, 100)

fn min_max_percent(value: i32) -> Percent:
    if value >= 0:
        lower = max(value, 0)
        bounded = min(lower, 100)
        return bounded
    return Percent.clamped(0)

fn clamp_ratio(value: f32) -> Ratio:
    return clamp(value, 0.0, 1.0)

fn min_max_ratio(value: NonNegative) -> Ratio:
    erased: f32 = value
    lower = max(erased, 0.0)
    bounded = min(lower, 1.0)
    return bounded

fn main():
    first: i32 = clamp_percent(42)
    second: i32 = min_max_percent(42)
    ratio: f32 = clamp_ratio(0.25)
    bounded_ratio: f32 = min_max_ratio(0.25)
    println(first + second - 42)
    println(ratio + bounded_ratio)
