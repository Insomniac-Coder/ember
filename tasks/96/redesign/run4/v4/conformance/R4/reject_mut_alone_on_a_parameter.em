#$ test: compile-fail
#$ rules: R4
#$ help: fn bump(ref mut n: int):
## R4: the word in front of a parameter's name says what the function may do with it; `mut` alone
## says "may change" but not whose value (owner's test; AN-27, E0113).

fn bump(mut n: int):    #$ error[E0113]: `mut n` on a parameter: to change the caller's `n` through `n`, write `ref mut n: int`, and call it `bump(ref mut a)`
    n += 1

fn main():
    a = 1
    bump(a)
    println(a)
