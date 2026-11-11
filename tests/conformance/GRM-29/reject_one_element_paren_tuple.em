#$ test: compile-fail
#$ rules: GRM-29
#$ error[E0100]: a tuple has two or more elements; write `e`
# `(e,)` is no tuple: a tuple has two or more elements.

fn main():
    t = (5,)
    println(t)
