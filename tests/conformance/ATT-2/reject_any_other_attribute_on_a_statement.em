#$ test: compile-fail
#$ rules: ATT-2
#$ help: only `@simd`, `@parallel`, `@unroll` and `@allow` are
# `[ATT-2]` — a statement may carry only `@parallel`, `@unroll`, `@simd` and `@allow`: any other
# attribute, even one that is in the table for a function, is `E0108`.

fn main():
    c = 3
    @inline                         #$ error[E0108]: `@inline` is not permitted on a statement
    for i in range(c):
        println(i)
    @deprecated("old", "new")       #$ error[E0108]: `@deprecated` is not permitted on a statement
    if c == 3:
        println(c)
    @noalloc                        #$ error[E0108]: `@noalloc` is not permitted on a statement
    while c > 0:
        c -= 1
