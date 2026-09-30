#$ test: run-pass
#$ rules: RNG-4, TYP-28, COST-3
#$ profiles: debug, release, shipping
#$ stdout: -1999 1007 -2000
#$ stdout: 3002
#$ stdout: -3997 3002 -3000
#$ stdout: 2998 -4003
#$ assert-c-count: contains("ember_panic_div_zero(") == 0
#$ assert-c-count: contains("ember_ck_floordiv_i64(") == 0
#$ assert-c-count: contains("ember_ck_floorrem_i64(") == 0
#$ assert-c-count: contains("_1 >> 2LL;") == 1
#$ assert-c-count: contains("_1 & 7LL;") == 1
#$ assert-c-count: contains(">> 63LL;") == 2
#$ assert-c-count: contains("ember_floordiv_i64(") == 1
#$ assert-c-count: contains("ember_floorrem_i64(") == 1
# `[RNG-4]`, `[COST-3]` — a floor `//` or `%` whose checks the facts remove
# is written the cheapest way that rounds down: by a power of two, a shift or
# a mask, for either sign (`-7 // 4 == -2`, `-7 % 8 == 1`); with both sides
# non-negative, C's unsigned operators; with only the divisor positive, C's
# operators corrected by the remainder's sign bit (`>> 63`), with no branch;
# otherwise the runtime's floor form, with no overflow flag. A divisor proved
# positive, through `and` as well, needs no zero check.

fn by_power(x: int) -> int:
    return (x // 4) * 1000 + x % 8

fn non_negative(x: int, d: int) -> int:
    if x >= 0 and d > 0:
        return (x // d) * 1000 + x % d
    return 0

fn any_sign(x: int, d: int) -> int:
    if d >= 1 and d <= 100:
        return (x // d) * 1000 + x % d
    return 0

fn negative_divisor(x: int, d: int) -> int:
    if d <= -2 and d >= -100:
        return (x // d) * 1000 + x % d
    return 0

fn main():
    println(by_power(-7), by_power(7), by_power(-8))
    println(non_negative(17, 5))
    println(any_sign(-17, 5), any_sign(17, 5), any_sign(-15, 5))
    println(negative_divisor(-17, -5), negative_divisor(17, -5))
