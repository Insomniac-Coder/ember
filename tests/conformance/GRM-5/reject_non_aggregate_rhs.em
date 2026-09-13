#$ test: compile-fail
#$ rules: GRM-5
#$ error[E2020]: destructuring requires a tuple or struct, found `i32`

fn main():
    a, b = 1

