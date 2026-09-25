#$ test: run-pass
#$ rules: TYP-9, TYP-29, STD-20
#$ profiles: debug, release, shipping
#$ stdout: 3.75 -0.75 3.375 0.6665 0.0 1.5 -1.5 2.25
#$ true false true
#$ 65500.0 inf 0.0 6e-08 2e-07
#$ 65500.0 -65500.0 0.000977 inf nan
#$ true -0.0 inf -inf false
#$ 1.5 1.500      1.5 2.25 9.997559e-02
# D-316 — `f16` is IEEE binary16 (IV.2): each operation is rounded once to
# it, ties to even (`[TYP-9]`), and prints as the fewest digits that read
# back as it. It was integer arithmetic on the truncated value.

fn main():
    a: f16 = 1.5
    b: f16 = 2.25
    println(a + b, a - b, a * b, a / b, a // b, a % b, -a, a ** 2.0)
    println(a < b, a == b, a != b)
    big = f16.MAX
    tiny: f16 = 0.00000006
    # 65519 rounds down to 65504; 65520 is halfway to 2^16 and rounds to
    # infinity; half the least subnormal rounds to zero, its even neighbour.
    println(big + 15.0, big + 16.0, tiny / 2.0, tiny, tiny * 3.0)
    println(f16.MAX, f16.MIN, f16.EPSILON, f16.INF, f16.NAN)
    z: f16 = 0.0
    println(z == -z, -z, 1.0 / z, -1.0 / z, z / z == z / z)
    x: f16 = 0.1
    println(f"{a} {a:.3f} {a:>8} {b!r} {x:e}")
