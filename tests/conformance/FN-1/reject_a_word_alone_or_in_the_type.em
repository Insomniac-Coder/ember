#$ test: compile-fail
#$ rules: FN-1, GRM-41
# `[FN-1]` — `mut` alone on a parameter is `E0113`, and `ref mut` written in a parameter's
# type is `E0117`.

fn a(mut n: int):    #$ error[E0113]: `mut` alone is no word for a parameter
    println(n)

fn c(n: ref mut int):    #$ error[E0117]
    println(n)

fn main():
    pass
