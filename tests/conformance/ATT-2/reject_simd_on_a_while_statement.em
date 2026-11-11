#$ test: compile-fail
#$ rules: ATT-2
# `[ATT-2]` — `@parallel`, `@unroll` and `@simd` go only on a `for` statement: `E0108`.

fn main():
    c = 3
    @simd                           #$ error[E0108]: `@simd` applies to a `for` loop
    while c > 0:
        c = c - 1
    println(c)
