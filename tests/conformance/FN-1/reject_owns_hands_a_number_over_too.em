#$ test: compile-fail
#$ rules: FN-1
# `[FN-1]` — `owns c: C` is handed over for every type, numbers included: the caller's name ends,
# and a later use is `E3040`.

fn eat(owns n: int):
    println(n)

fn main():
    x = 1
    eat(owns x)
    println(x)    #$ error[E3040]: `x` was handed over to `eat`
