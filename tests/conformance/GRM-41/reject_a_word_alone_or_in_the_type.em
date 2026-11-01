#$ test: compile-fail
#$ rules: GRM-41, FN-1
## `[GRM-41]` — `mut` alone on a parameter is `E0113`, and a mode written in the parameter's type is
## `E0117`; each help gives the line with the word on the name.

fn bump(mut n: int):    #$ error[E0113]: `mut` alone is no word for a parameter: one the function may change is `ref mut`
    n += 1

fn grow(c: ref mut int):    #$ error[E0117]: the mode goes on the name, not in its type
    c += 1

fn main():
    k = 1
    bump(ref mut k)
    grow(ref mut k)
    println(k)
