#$ test: parse-fail
#$ rules: GRM-20, ATT-1, ATT-2
## `[GRM-20]` — the attributes before a statement are `@parallel`, `@unroll`,
## `@simd` and `@allow`.

fn main():
    @inline    #$ error[E0104]: `@inline` is not permitted on a statement
    x = 1
    println(x)
