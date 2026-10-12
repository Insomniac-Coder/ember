#$ test: compile-fail
#$ rules: GRM-36, BRW-1
## `[GRM-36]` — `ref e` borrows the place `e`; a call's result is a value, not
## a place, and is `E0111`, whose help binds it to a local. It was `E2140`,
## the code for assigning to a value (D-581).

fn five() -> int:
    return 5

fn main():
    r = ref five()    #$ error[E0111]: only a place can be borrowed
    println(r)
