#$ test: run-pass
#$ rules: TYP-37
#$ profiles: debug, release, shipping
#$ stdout: [-inf, -1.0, -0.0, 0.0, 2.5, 3.0, nan]
#$ -2.25 1.5 2.25 1.0
#$ true false true
# D-316 — `f16` sorts by IEEE totalOrder and compares with IEEE `==`, as
# every float does (`[TYP-37]`); on the stored bits `-0.0` sorted above
# `3.0`.

fn main():
    xs: Array[f16] = [3.0, -1.0, 2.5, f16.NAN, -0.0, 0.0, -f16.INF]
    xs.sort()
    println(xs)
    a: f16 = 1.5
    b: f16 = -2.25
    println(min(a, b), max(a, b), abs(b), clamp(a, 0.0, 1.0))
    z: f16 = 0.0
    println(z == -z, f16.NAN == f16.NAN, f16.NAN != f16.NAN)
