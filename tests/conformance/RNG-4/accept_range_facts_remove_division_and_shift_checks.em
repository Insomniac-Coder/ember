#$ test: run-pass
#$ rules: RNG-4, TYP-8, TYP-10
#$ profiles: debug, release, shipping
#$ stdout: 14 0 0
#$ stdout: 8 9223372036854775808
#$ assert-c-count: contains("ember_panic_div_zero(") == 0
#$ assert-c-count: contains("ember_panic_overflow(") == 0
# `[RNG-4]` — `parts > 0` and `not (d <= 0 or d > 100)` each rule out a zero
# divisor, and a positive divisor rules out `MIN // -1`; `k % 64` is below
# 64, the width of the shifted `u64`. No division, overflow or shift check
# is left.

fn share(total: int, parts: int) -> int:
    if parts > 0:
        return total // parts
    return 0

fn share_bounded(total: int, d: int) -> int:
    if d <= 0 or d > 100:
        return 0
    return total // d

fn bit(k: int) -> u64:
    s = k % 64
    return (1 as u64) << (s as u64)

fn main():
    println(share(100, 7), share(100, 0), share_bounded(100, 0))
    println(bit(3), bit(-1))
