#$ test: compile-fail
#$ rules: FN-1
#$ help: fn bump(ref mut n: int):
## design rules: R4
## R4: the word in front of a parameter's name says what the function may do with it; `mut` alone
## says "may change" but not whose value (owner's test; AN-27, E0113). The message names the caller's
## variable, `a`, not the parameter (step 3's readability check, F-13).

fn bump(mut n: int):    #$ error[E0113]: `mut n` on a parameter: to change the caller's `a` through `n`, write `ref mut n: int`, and call it `bump(ref mut a)`
    n += 1

fn main():
    a = 1
    bump(a)
    println(a)
