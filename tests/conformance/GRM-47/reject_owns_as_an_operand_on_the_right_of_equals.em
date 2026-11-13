#$ test: compile-fail
#$ rules: GRM-47, GRM-43
## `[GRM-47]` — on the right of `=` the message says that `n` would end before the multiplication and
## that `=` hands the product over with no word.

fn main():
    n = 4
    b = owns n * 2    #$ error[E0119]: `owns` here is an operand of `*`, which only reads `n`
                      #$ help: write `b = n * 2`
    println(b)
