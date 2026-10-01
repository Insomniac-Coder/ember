#$ test: run-pass
#$ rules: TYP-8, CG-C-1
#$ profiles: debug, release, shipping
#$ stdout: 200000 59999700000
#$ stdout: -2147483648 -9223372036854775808 24464 -32768
# D-446 — wrapping arithmetic is two's-complement and never undefined
# behaviour in the generated C (`[TYP-8]`, `[CG-C-1]`). A wrapping signed
# `*` reached C as a signed multiply: gcc -O2 took `key(i)` in a loop as never
# overflowing and computed other keys than the same call elsewhere, so the
# lookups below missed (an overflow panic in release, a hang in shipping).

@overflow(wrap)
fn key(i: int) -> int:
    return (i * 0x5851F42D4C957F2D) >> 8

@overflow(wrap)
fn edges() -> (i32, int, i16, i16):
    big: i32 = 2147483647
    least: int = -9223372036854775807 - 1
    small: i16 = 300
    low: i16 = -32768
    return (big + 1, -least, small * small, -low)

fn main():
    n = 200000
    m: Map[int, int] = {}
    for i in 0..n:
        m.insert(key(i), i)
    total = 0
    for round in 0..3:
        for i in 0..n:
            total += m[key(i)]
    println(len(m), total)
    (a, b, c, d) = edges()
    println(a, b, c, d)
