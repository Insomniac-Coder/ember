#$ test: compile-fail
#$ rules: GRM-5
#$ error[E2020]: `(i32, i32)` has 2 fields, but this destructuring has 3 targets

fn main():
    a, b, c = (1, 2)

