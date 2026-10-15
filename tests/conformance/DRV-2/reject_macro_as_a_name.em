#$ test: compile-fail
#$ rules: DRV-2, LEX-14
#$ error[E0005]: `macro` is reserved
#$ error[E0005]: `macro` is reserved
#$ help: write `r#macro` to use the word as an identifier
# `[DRV-2]` — `macro` is reserved for user-defined derives, so it is no name; `r#macro` is.

fn main():
    macro = 1
    println(macro)
