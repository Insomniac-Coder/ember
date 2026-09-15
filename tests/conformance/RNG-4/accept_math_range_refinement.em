#$ test: run-pass
#$ rules: RNG-4, RNG-10, MOD-1
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ 0.5

from std.math import clamp_i32, min_i32, max_i32, clamp_f32, min_f32, max_f32

type Percent = i32 in 0 ..= 100
type Ratio = f32 in 0.0 ..= 1.0
type NonNegative = f32 in 0.0 ..= 3.0

fn clamp_percent(value: i32) -> Percent:
    return clamp_i32(value, 0, 100)

fn min_max_percent(value: i32) -> Percent:
    if value >= 0:
        lower = max_i32(value, 0)
        bounded = min_i32(lower, 100)
        return bounded
    return Percent.clamped(0)

fn clamp_ratio(value: f32) -> Ratio:
    return clamp_f32(value, 0.0, 1.0)

fn min_max_ratio(value: NonNegative) -> Ratio:
    lower = max_f32(value, 0.0)
    bounded = min_f32(lower, 1.0)
    return bounded

fn main():
    first: i32 = clamp_percent(42)
    second: i32 = min_max_percent(42)
    ratio: f32 = clamp_ratio(0.25)
    bounded_ratio: f32 = min_max_ratio(0.25)
    println(first + second - 42)
    println(ratio + bounded_ratio)
