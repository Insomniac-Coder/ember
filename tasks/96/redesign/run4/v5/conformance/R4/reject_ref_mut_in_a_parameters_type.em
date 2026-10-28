#$ test: compile-fail
#$ rules: FN-1
#$ help: fn bump(ref mut n: int):
## design rules: R4
## R4: the mode goes on the parameter's name, not in its type; the type says only what the caller's
## value is (AN-27, E0117).

fn bump(n: ref mut int):    #$ error[E0117]: the mode goes on the name: write `ref mut n: int`
    n += 1

fn main():
    a = 1
    bump(ref mut a)
    println(a)
