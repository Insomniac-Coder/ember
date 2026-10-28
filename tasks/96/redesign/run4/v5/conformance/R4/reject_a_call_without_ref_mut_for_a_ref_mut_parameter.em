#$ test: compile-fail
#$ rules: FN-2a
#$ help: bump(ref mut a)
## design rules: R4
## R4, verdict 1: a call says `ref mut` before an argument the callee may change, with the same words
## as the parameter; a missing `ref mut` is refused (AN-27).

fn bump(ref mut n: int):
    n += 1

fn main():
    a = 1
    bump(a)    #$ error[E3028]: `bump` may change `a` (its first line says `ref mut n: int`); say so at the call: `bump(ref mut a)`
    println(a)
