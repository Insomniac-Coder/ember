#$ test: run-pass
#$ rules: STD-20, TYP-8, TYP-10, TYP-28, TYP-30
#$ profiles: debug, release, shipping
#$ stdout: None Some(2147483646) Some(255) None
#$ -2147483648 4 2147483647 255
#$ (-2147483648, true) (255, true) 0
#$ None -128 127 (-128, true)
#$ Some(-4) None Some(2) None -3 -1
#$ None -9223372036854775808 9223372036854775807 (-9223372036854775808, true) Some(0)
#$ 7 -1 0 1 1 250
#$ 81 Some(4611686018427387904) None None 0 9223372036854775807 -9223372036854775808 (-9223372036854775808, false)
#$ Some(128) None 2 (1, true) 0
#$ 3 25 3 8 8 16
#$ false true false false
#$ 128 1 1 2048
#$ 128 None 0 3 100 (-1, false)
# ODR-039, `[STD-20]` — the integer methods. For each operator (`add`, `sub`,
# `mul`, `floordiv`, `rem`, `pow`, `neg`, and the shifts): `checked_` gives
# `None` where the operator would panic, `wrapping_` the result modulo 2^N,
# `saturating_` the nearest bound, `overflowing_` the wrapped result and
# whether it wrapped. A shift's wrapping form takes the amount modulo the
# width; `(-2) ** 63` is `i64.MIN` exactly and does not wrap. The bit counts
# are over the type's width, so a zero `u8` has 8 of each.

fn main():
    a: i32 = 2147483647
    b: u8 = 250
    println(a.checked_add(1), a.checked_add(-1), b.checked_add(5), b.checked_add(6))
    println(a.wrapping_add(1), b.wrapping_add(10), a.saturating_add(5), b.saturating_add(10))
    println(a.overflowing_add(1), b.overflowing_sub(251), b.saturating_sub(251))
    c: i8 = -128
    println(c.checked_neg(), c.wrapping_neg(), c.saturating_neg(), c.overflowing_neg())
    n: int = -7
    println(n.checked_floordiv(2), n.checked_floordiv(0), n.checked_rem(3), n.checked_rem(0), n.div_trunc(2), n.rem_trunc(2))
    m: i64 = -9223372036854775808
    println(m.checked_floordiv(-1), m.wrapping_floordiv(-1), m.saturating_floordiv(-1), m.overflowing_floordiv(-1), m.checked_rem(-1))
    println(n.abs(), n.signum(), 0.signum(), 5.signum(), b.signum(), b.abs())
    println(3.pow(4), 2.checked_pow(62), 2.checked_pow(63), 2.checked_pow(-1), 2.wrapping_pow(64), 3.saturating_pow(50), (-3).saturating_pow(41), (-2).overflowing_pow(63))
    one: u8 = 1
    println(one.checked_shl(7), one.checked_shl(8), one.wrapping_shl(9), one.overflowing_shl(8), one.wrapping_shr(-1))
    x: u32 = 0b1011000
    println(x.count_ones(), x.leading_zeros(), x.trailing_zeros(), 0u8.leading_zeros(), 0u8.trailing_zeros(), (-1i16).count_ones())
    println(x.is_power_of_two(), 64.is_power_of_two(), 0.is_power_of_two(), (-4).is_power_of_two())
    println(x.next_power_of_two(), 1.next_power_of_two(), 0.next_power_of_two(), 1025u16.next_power_of_two())
    big: u128 = 340282366920938463463374607431768211455
    wide: i128 = 170141183460469231731687303715884105727
    println(big.count_ones(), big.checked_add(1), big.leading_zeros(), (big >> 3).leading_zeros(), (1u128 << 100).trailing_zeros(), (wide.wrapping_mul(2) + 1).overflowing_pow(3))
