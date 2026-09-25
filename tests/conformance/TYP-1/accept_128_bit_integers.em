#$ test: run-pass
#$ rules: TYP-1, TYP-8, TYP-10, TYP-28, TYP-30, LEX-19
#$ profiles: debug, release, shipping
#$ stdout:
#$ -170141183460469231731687303715884105728 170141183460469231731687303715884105727
#$ 340282366920938463463374607431768211455 170141183460469231731687303715884105728
#$ 128 -134217728 -170141183460469231731687303715884105728
#$ 255 170141183460469231731687303715884105727 0
#$ -24691357802469135780246 -1763668414462081127161 4 1763668414462081127160 -3
#$ 1267650600228229401496703205376 340282366920938463463374607431768211454
#$ true false true false
#$ max five zero other
#$ [-170141183460469231731687303715884105728, -3, 0, 5, 170141183460469231731687303715884105727]
#$ -4 4
#$ true Pair(a=-1, b=340282366920938463463374607431768211455)
#$ Some(-170141183460469231731687303715884105728)
#$ 7fffffffffffffffffffffffffffffff 1000000000000000000000000000000000000000000000000000000000000000
#$ [     -170141183460469231731687303715884105728]|170,141,183,460,469,231,731,687,303,715,884,105,727|-0000042
#$ 1.701412e+38 170141183460469231731687303715884105727
# D-272 — `i128` and `u128` reach C: literals, arithmetic (floor division and
# modulo, `**`), bit operations (`>>` arithmetic on a signed value), order,
# `match`, sorting, equality and printing, including through a struct, an
# `Option` and f-string specs. MSVC has no 128-bit integer, so the runtime
# carries both types as two 64-bit halves there, and every operation on one
# is a runtime helper; the answers are the same on every compiler.

struct Pair:
    a: i128
    b: u128

fn describe(x: i128) -> str:
    match x:
        0 => return "zero"
        170141183460469231731687303715884105727 => return "max"
        5 => return "five"
        _ => return "other"

fn main():
    lo: i128 = -170141183460469231731687303715884105727 - 1
    hi: i128 = 170141183460469231731687303715884105727
    println(lo, hi)
    u: u128 = 1
    println(340282366920938463463374607431768211455u128, u << 127)
    println((u << 127) >> 120, lo >> 100, ~hi)
    println(hi & 255, hi | 1, hi ^ hi)
    x: i128 = -12345678901234567890123
    println(x * 2, x // 7, x % 7, -x // 7, x % -7)
    println(2i128 ** 100, (u << 127) - 1 + (u << 127) - 1)
    println(lo < hi, hi <= lo, x == x, x != x)
    println(describe(hi), describe(5), describe(0), describe(7))
    xs: Array[i128] = [5, -3, hi, lo, 0]
    xs.sort()
    println(xs)
    println(min(3i128, -4i128), max(3u128, 4u128))
    p = Pair(a=-1, b=340282366920938463463374607431768211455)
    q = Pair(a=-1, b=340282366920938463463374607431768211455)
    println(p == q, p)
    o: Option[i128] = Some(lo)
    println(o)
    println(f"{hi:x} {u << 63:b}")
    println(f"[{lo:>45}]|{hi:,}|{-42i128:+08}")
    println(f"{hi:e}", hi.to_string())
