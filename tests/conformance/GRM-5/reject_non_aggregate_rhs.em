#$ test: compile-fail
#$ rules: GRM-5
#$ error[E2020]: destructuring requires a tuple, a struct or a fixed array, found `i64`

fn main():
    a, b = 1

