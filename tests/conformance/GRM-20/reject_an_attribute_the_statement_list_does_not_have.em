#$ test: compile-fail
#$ rules: GRM-20, ATT-2
# `[GRM-20]`, `[ATT-2]` — an attribute outside the four a statement may carry is `E0108`.

fn main():
    @inline                         #$ error[E0108]: `@inline` is not permitted on a statement
    for i in 0..3:
        println(i)
