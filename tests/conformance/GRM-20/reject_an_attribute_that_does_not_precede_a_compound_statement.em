#$ test: compile-fail
#$ rules: GRM-20, ATT-2
# `[GRM-20]` — the attributes before a statement are limited to `@parallel`, `@unroll`, `@simd` and
# `@allow`; the ones that are listed go before a compound statement, and `@simd` before an
# assignment has nothing to describe.

fn main():
    x = 3
    @simd                           #$ error[E0108]: `@simd` must precede a compound statement
    x = 4
    println(x)
