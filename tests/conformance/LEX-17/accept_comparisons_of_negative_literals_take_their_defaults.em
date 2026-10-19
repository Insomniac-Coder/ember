#$ test: run-pass
#$ rules: LEX-16, LEX-17, TYP-4
#$ profiles: debug, development, release
#$ stdout: true false true
#$ stdout: true false
#$ stdout: false true
#$ stdout: false true true
# `[LEX-16]`, `[LEX-17]` (D-545) — an untyped literal is never left open: a comparison's value is a
# `bool`, so its two untyped operands take their default types there, `f64` for floats and `int`
# for integers. Left untyped, `-1.5 < -1.0` compared two 32-bit integers in C (-1 and -1) and was
# false, and `-1.5 == -1.0` was true.

fn main():
    println(-1.5 < -1.0, -1.5 == -1.0, -1.5 < -1.4)
    println(-3000000000 < -2999999999, -3000000000 == -2999999999)
    println(-0.0 < 0.0, -0.0 == 0.0)
    println(1.5 < 1.0, 1.5 > 1.4, 2 > 1)
