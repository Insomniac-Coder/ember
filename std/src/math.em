#! language "0.8.3"
## `std.math` — Part XV's scalar mathematics.
##
## `[STD-1]` — this module MUST be `@noalloc`-clean except where documented to
## allocate; nothing here allocates.

## The smaller of two values.
##
## `[RNG-5a]` calls `min`, `max` and `clamp` the **range-preserving clamp
## family**: applied to operands of one range type, or to a range value and
## constants of its representation lying within its range, the result is that
## range type rather than the representation. That exception is the compiler's
## to apply; this is the ordinary function it applies it to.
pub fn min_f32(a: f32, b: f32) -> f32:
    if a < b:
        return a
    return b

pub fn max_f32(a: f32, b: f32) -> f32:
    if a > b:
        return a
    return b

## `[RNG-3a]` defines `T.clamped(v)` as `min(max(v, lo), hi)`; this is that
## function written out, for a value that is not of a range type.
pub fn clamp_f32(v: f32, lo: f32, hi: f32) -> f32:
    return min_f32(max_f32(v, lo), hi)

pub fn min_i32(a: i32, b: i32) -> i32:
    if a < b:
        return a
    return b

pub fn max_i32(a: i32, b: i32) -> i32:
    if a > b:
        return a
    return b

pub fn clamp_i32(v: i32, lo: i32, hi: i32) -> i32:
    return min_i32(max_i32(v, lo), hi)

## `[STD-3]` — "fused multiply-add is expressible as an explicit, IEEE-defined
## operation **with no float-control attribute at all**". `fma` itself needs a
## backend intrinsic (`fmaf`), which is not built; this records the shape the
## rule requires so the name is not taken by something weaker.
pub fn lerp_f32(a: f32, b: f32, t: f32) -> f32:
    return a + (b - a) * t
