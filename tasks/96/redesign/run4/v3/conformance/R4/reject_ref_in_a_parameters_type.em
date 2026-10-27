#$ test: compile-fail
#$ rules: R4
#$ help: fn show(x: int):
## R4: `ref` in a parameter's type is refused; a plain parameter already reads the caller's value
## (AN-27, E0117).

fn show(x: ref int):    #$ error[E0117]: the mode goes on the name, and a plain parameter already reads the caller's value without copying it: write `x: int`
    println(x)

fn main():
    show(3)
