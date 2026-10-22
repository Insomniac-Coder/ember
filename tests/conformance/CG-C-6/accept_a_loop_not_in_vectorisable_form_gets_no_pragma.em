#$ test: run-pass
#$ rules: CG-C-6, SIMD-5, SIMD-3
#$ profiles: release
#$ stdout: 11
#$ 3
#$ assert-c: !contains("EMBER_VECTORISE")
# `[CG-C-6]`: the pragma is written only where `[SIMD-5]` holds, since GCC's and MSVC's state that
# iterations share no memory. An indirect index (`xs[idx[i]]`, not the counter plus a constant) and
# an early `return` (a second exit) each keep a loop out of vectorisable form, so neither gets it:
# `@simd` asks and reports, and here reports no.

fn gathered(xs: Span[int], idx: Span[int]) -> int:
    s = 0
    @simd
    for i in 0..idx.len():
        s += xs[idx[i]]
    return s

fn first_at_least(xs: Span[int], limit: int) -> int:
    @simd
    for i in 0..xs.len():
        if xs[i] >= limit:
            return i
    return -1

fn main():
    a = [1, 2, 3, 4]
    println(gathered(a, [3, 3, 1, 0]))
    println(first_at_least(a, 4))
