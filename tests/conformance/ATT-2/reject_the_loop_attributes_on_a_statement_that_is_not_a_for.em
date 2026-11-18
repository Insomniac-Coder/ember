#$ test: compile-fail
#$ rules: ATT-2
# `[ATT-2]` — `@parallel`, `@unroll` and `@simd` go only on a `for` statement: `E0108`, whatever
# other compound statement they stand before.

fn main():
    c = 3
    @parallel                       #$ error[E0108]: `@parallel` applies to a `for` loop
    while c > 0:
        c = c - 1
    @unroll(4)                      #$ error[E0108]: `@unroll` applies to a `for` loop
    if c == 0:
        println(c)
    @simd                           #$ error[E0108]: `@simd` applies to a `for` loop
    match c:
        0:
            println(c)
        _:
            pass
