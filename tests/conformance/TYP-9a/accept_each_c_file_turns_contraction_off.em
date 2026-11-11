#$ test: run-pass
#$ rules: TYP-9a, CG-C-11
#$ profiles: debug, development, release
#$ assert-c: contains("#pragma STDC FP_CONTRACT OFF")
#$ stdout: 0.0
# `[TYP-9a]` — contraction is off by default and the implementation turns it off in the host C
# compiler (`[CG-C-11]`): `a * b + c` is two roundings, never a fused multiply-add the C compiler
# chose. With `a = b = 1 + 2^-12` and `c = -(1 + 2^-11)` the product rounds to `1 + 2^-11` in `f32`,
# so the sum is 0; fused it would keep `2^-24`.

fn main():
    a: f32 = 1.0 + 1.0 / 4096.0
    b = a
    c: f32 = -(1.0 + 1.0 / 2048.0)
    println(a * b + c)
