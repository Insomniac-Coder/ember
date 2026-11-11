#$ test: compile-fail
#$ rules: GRM-29
#$ error[E0100]: a tuple has two or more elements; write `e`
# `(p,)` in a pattern.

fn main():
    a = 0
    (a,) = 1
    println(a)
