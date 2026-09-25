#$ test: run-pass
#$ rules: TYP-9, TYP-9a, CG-C-11, STD-3
#$ profiles: debug, release, shipping
#$ assert-c: contains("#pragma STDC FP_CONTRACT OFF")
#$ assert-c: contains("#pragma fp_contract(off)")
#$ stdout: 0.0 5.9604645e-08 5.9604645e-08
#$ 0.0 5.551115123125783e-17
# D-325 — `a * b + c` is two roundings, never a fused multiply-add the C
# compiler chose (`[TYP-9]`). With `a = b = 1 + 2^-12` and `c = -(1 + 2^-11)`,
# `a * b` is `1 + 2^-11 + 2^-24`, which `f32` rounds to `1 + 2^-11`, so the
# sum is 0; fused, it keeps the `2^-24`. Each C file turns contraction off
# (`[CG-C-11]`): clang's pragma, MSVC's, and gcc's `-ffp-contract=off`.
# `mul_add` and `math.fma` are the one rounding when it is wanted (`[STD-3]`).

import math

fn main():
    a: f32 = 1.0 + 1.0 / 4096.0
    b = a
    c: f32 = -(1.0 + 1.0 / 2048.0)
    println(a * b + c, a.mul_add(b, c), math.fma(a, b, c))
    x = 1.0 + 1.0 / 134217728.0
    y = -(1.0 + 1.0 / 67108864.0)
    println(x * x + y, x.mul_add(x, y))
