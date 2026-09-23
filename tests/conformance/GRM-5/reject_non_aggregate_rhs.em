#$ test: compile-fail
#$ rules: GRM-5
#$ error[E2020]: destructuring requires a tuple or struct, found `i64`

fn main():
    a, b = 1

