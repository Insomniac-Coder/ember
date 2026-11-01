#$ test: compile-fail
#$ rules: OWN-9, GRM-15
# `[OWN-9]` — `=` copies a number, so `b = owns n` is how `=` hands one over: `n` ends there.

fn main():
    n = 5
    b = owns n
    println(b)
    println(n)  #$ error[E3040]
