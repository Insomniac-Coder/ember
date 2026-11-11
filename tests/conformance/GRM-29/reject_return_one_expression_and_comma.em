#$ test: compile-fail
#$ rules: GRM-29
#$ error[E0100]: a tuple has two or more elements; write `e`
# One expression and a comma after `return`.

fn one() -> int:
    return 5,

fn main():
    println(one())
