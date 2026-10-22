#$ test: run-pass
#$ rules: RNG-3a, RNG-1
#$ profiles: debug, development, release
#$ stdout: 100 -2147483648 7
#$ 0 -9223372036854775808 -5
## `[RNG-3a]` — `T.clamped(v)` is `min(max(v, lo), hi)` when `lo` is the representation's least
## value too: a value above `hi` is `hi`, the least value is itself, one inside is unchanged
## (D-892: C read `-2147483648` as a minus applied to an unsigned literal, so a clamp compared
## against it unsigned and gave `lo` for `500`).

type Upper = i32 in -2147483648 ..= 100
type Lower = i64 in -9223372036854775808 ..= 0

fn upper(v: i32) -> Upper:
    return Upper.clamped(v)

fn lower(v: i64) -> Lower:
    return Lower.clamped(v)

fn main():
    least32: i32 = -2147483647 - 1
    least64: i64 = -9223372036854775807 - 1
    a: i32 = upper(500)
    b: i32 = upper(least32)
    c: i32 = upper(7)
    println(a, b, c)
    d: i64 = lower(5)
    e: i64 = lower(least64)
    f: i64 = lower(-5)
    println(d, e, f)
