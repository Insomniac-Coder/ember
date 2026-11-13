#$ test: run-pass
#$ rules: CG-C-1, TYP-8
#$ profiles: debug, development
#$ stdout: 8 2 42
#$ assert-c: contains("ember_ck_add_i64(")
#$ assert-c: contains("ember_ck_floordiv_i64(")
#$ assert-c: contains("ember_panic_div_zero(")
#$ assert-c: contains("ember_panic_overflow("shift"")
# `[CG-C-1]` — the emitted C has no undefined behaviour: checked signed arithmetic calls the checked
# helpers of `ember_rt.h` (the compiler's `__builtin_add_overflow` and friends on Clang and GCC, exact
# checks of their own on MSVC), a
# division is checked for a zero divisor and for `MIN // -1` before the C operator runs, and a
# shift is range-checked before it executes.

fn main():
    xs: Array[int] = [3, 5, 2]
    a = xs[0] + xs[1]
    b = a // (xs[2] * 2)
    c = a << xs[1]
    println(a, b, c // 6)
