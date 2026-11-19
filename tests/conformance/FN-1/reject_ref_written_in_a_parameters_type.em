#$ test: compile-fail
#$ rules: FN-1, GRM-41
# `[FN-1]` — `ref` or `ref mut` written in a parameter's type is `E0117`; the message gives the
# line with the word on the name. Only the top of the type: a `ref` inside one is a view type.

fn a(n: ref int):    #$ error[E0117]: the mode goes on the name, not in its type
    println(n)

fn b(n: ref mut int):    #$ error[E0117]: the mode goes on the name, not in its type
    println(n)

fn main():
    pass
