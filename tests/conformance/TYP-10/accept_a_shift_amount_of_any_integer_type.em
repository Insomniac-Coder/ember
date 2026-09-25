#$ test: run-pass
#$ rules: TYP-10
#$ profiles: debug, release, shipping
#$ stdout: 8 50 144 1267650600228229401496703205376 -64
#$ -9223372036854775808 2
# D-308 — `[TYP-10]`: `a << n` and `a >> n` take an `n` of any integer type,
# and the result has `a`'s type; bits shifted out are discarded (`200 << 1`
# is 144 in a `u8`). `>>` of a signed value is arithmetic. Under
# `@overflow(wrap)` the amount is masked instead of checked, a negative one
# too.

fn shift(x: int, n: i32) -> int:
    return x << n

@overflow(wrap)
fn wrapped(x: int, n: i8) -> int:
    return x << n

fn main():
    a: u8 = 200
    w: i128 = 1
    down: i16 = -256
    println(shift(1, 3), a >> 2u64, a << 1u128, w << 100u8, down >> 2u8)
    println(wrapped(1, -1), wrapped(1, 65))
